package com.serbekun.mgdsw.resources;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Files;
import java.nio.file.InvalidPathException;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import com.serbekun.mgdsw.config.configs.ConfigResources;

/**
 * Creates the runtime data directories and writes the packaged template resources
 * to disk on first run.
 *
 * <p>Resources are split into three independent roots, one per {@link ResourceType}:
 *
 * <ul>
 *   <li>{@code resources.staticBasePath} — templates and static assets served by
 *       nginx (HTML, CSS, JS, images, partials);</li>
 *   <li>{@code resources.contentBasePath} — operator-managed metadata that can
 *       change without rebuilding the server (for example, map YAML);</li>
 *   <li>{@code resources.sharedBasePath} — large shared files such as Minecraft
 *       map ZIP archives.</li>
 * </ul>
 *
 * <p>Every root and every directory declared by {@link ResourcesBasePath} is created
 * on startup. Only directories with {@link UnpackPolicy#PACKAGED} are copied from the
 * JAR, and each one is copied into the root of its own {@link ResourceType}. Disk-only
 * directories are created empty; their contents are supplied by the operator.
 *
 * <p>The unpacker is deliberately conservative:
 *
 * <ul>
 *   <li>a file that already exists on disk is never overwritten, so an edited template
 *       or a dropped-in asset survives a re-run;</li>
 *   <li>the marker carries the build version, so upgrading the JAR re-runs the copy and
 *       fills in templates that are new in that version while still leaving existing files
 *       alone;</li>
 *   <li>the marker lives in the static root only, because only packaged templates are
 *       versioned with the JAR. Content and shared files are owned by the operator.</li>
 * </ul>
 */
public class ResourcesUnpacker {

    private static final Logger log = LoggerFactory.getLogger(ResourcesUnpacker.class);

    /** Marker file, relative to the static root, recording the unpacked version. */
    public static final String MARKER_FILE = ".unpacked";

    private final ResourcesLoader loader;
    private final String version;

    /**
     * Creates an unpacker.
     *
     * @param loader  the loader that reads the packaged classpath resources
     * @param version the build version recorded in the marker
     */
    public ResourcesUnpacker(ResourcesLoader loader, String version) {
        this.loader = loader;
        this.version = version;
    }

    /**
     * Ensures the configured roots and their subdirectories exist, then unpacks the
     * packaged templates when the static copy is missing or from another version.
     *
     * @param configResources the configured static, content and shared roots
     * @return {@code true} when the templates were (re)written, {@code false} when the
     *         static copy was already current
     * @throws IllegalStateException when a root is blank, nested inside another root,
     *                               or the directories cannot be prepared
     */
    public boolean unpack(ConfigResources configResources) {
        if (configResources == null) {
            throw new IllegalStateException("configResources is required");
        }

        Path staticRoot = root(configResources.staticBasePath(), "resources.staticBasePath");
        Path contentRoot = root(configResources.contentBasePath(), "resources.contentBasePath");
        Path sharedRoot = root(configResources.sharedBasePath(), "resources.sharedBasePath");

        requireSeparate(staticRoot, contentRoot, sharedRoot);

        Path marker = staticRoot.resolve(MARKER_FILE);
        String unpackedVersion = Files.isRegularFile(marker) ? readMarker(marker) : null;
        boolean current = version.equals(unpackedVersion);

        if (current) {
            log.info("Resources already unpacked at {} (version {})", staticRoot, version);
        } else if (unpackedVersion != null) {
            log.info("Resources at {} are from version {}, updating to {}",
                    staticRoot, unpackedVersion.isBlank() ? "unknown" : unpackedVersion, version);
        }

        int copied = 0;
        try {
            createDirectories(staticRoot, contentRoot, sharedRoot);

            for (ResourcesBasePath dir : ResourcesBasePath.directoriesToCreate()) {
                Path dirRoot = rootFor(dir.getResourceType(), staticRoot, contentRoot, sharedRoot);
                createDirectory(dirRoot, dir.getBasePath());
            }

            if (!current) {
                for (ResourcesBasePath dir : ResourcesBasePath.packagedDirectories()) {
                    Path dirRoot = rootFor(dir.getResourceType(), staticRoot, contentRoot, sharedRoot);
                    copied += copyDirectory(dirRoot, dir.getBasePath());
                }
                Files.writeString(marker, version + System.lineSeparator(), StandardCharsets.UTF_8,
                        StandardOpenOption.CREATE,
                        StandardOpenOption.WRITE,
                        StandardOpenOption.TRUNCATE_EXISTING);
            }
        } catch (IOException e) {
            throw new IllegalStateException(
                    "Failed to prepare resource directories " + staticRoot + ", " + contentRoot
                            + " and " + sharedRoot, e);
        }

        if (!current) {
            log.info("Unpacked {} template file(s) into {} (version {})", copied, staticRoot, version);
        }
        return !current;
    }

    /**
     * Resolves and validates one configured root.
     *
     * @param value the configured path
     * @param key   the config key, used in the error message
     * @return an absolute, normalized path
     */
    private static Path root(String value, String key) {
        if (value == null || value.isBlank()) {
            throw new IllegalStateException(key + " must not be blank");
        }
        try {
            return Path.of(value).toAbsolutePath().normalize();
        } catch (InvalidPathException e) {
            throw new IllegalStateException(key + " is not a valid path: " + value, e);
        }
    }

    /**
     * Rejects equal or nested roots. The static root is served publicly, so content
     * and shared data must never live inside it.
     */
    private static void requireSeparate(Path staticRoot, Path contentRoot, Path sharedRoot) {
        requireSeparate(staticRoot, contentRoot, "resources.staticBasePath", "resources.contentBasePath");
        requireSeparate(staticRoot, sharedRoot, "resources.staticBasePath", "resources.sharedBasePath");
        requireSeparate(contentRoot, sharedRoot, "resources.contentBasePath", "resources.sharedBasePath");
    }

    private static void requireSeparate(Path first, Path second, String firstName, String secondName) {
        if (first.equals(second) || first.startsWith(second) || second.startsWith(first)) {
            throw new IllegalStateException(firstName + " and " + secondName
                    + " must be separate, non-nested directories: " + first + " vs " + second);
        }
    }

    private static Path rootFor(ResourceType type, Path staticRoot, Path contentRoot, Path sharedRoot) {
        return switch (type) {
            case STATIC -> staticRoot;
            case CONTENT -> contentRoot;
            case SHARED -> sharedRoot;
        };
    }

    private static void createDirectories(Path... roots) throws IOException {
        for (Path root : roots) {
            Files.createDirectories(root);
        }
    }

    private static void createDirectory(Path root, String dir) throws IOException {
        Path target = root.resolve(dir).normalize();
        if (!target.startsWith(root)) {
            throw new IllegalStateException("Resource directory escapes its root: " + dir);
        }
        Files.createDirectories(target);
    }

    /**
     * Copies every file of one packaged directory that is not already on disk.
     *
     * @param root the destination root for this resource type
     * @param dir  the packaged directory, with a trailing slash
     * @return how many files were written
     */
    private int copyDirectory(Path root, String dir) {
        int copied = 0;
        for (String path : loader.listClasspath(dir)) {
            Path target = root.resolve(path).normalize();
            if (!target.startsWith(root)) {
                log.warn("Refusing to unpack outside the resource root: {}", path);
                continue;
            }
            byte[] data = loader.loadFromClasspath(path);
            if (data == null) {
                continue;
            }
            try {
                Path parent = target.getParent();
                if (parent != null) {
                    Files.createDirectories(parent);
                }
                // CREATE_NEW both skips existing files and refuses to follow a symlink
                // planted at the destination.
                Files.write(target, data, StandardOpenOption.CREATE_NEW, StandardOpenOption.WRITE);
                copied++;
            } catch (FileAlreadyExistsException e) {
                log.debug("Keeping existing resource on disk: {}", target);
            } catch (IOException e) {
                throw new IllegalStateException("Failed to unpack " + path + " into " + root, e);
            }
        }
        return copied;
    }

    /**
     * Reads the version recorded in the marker.
     *
     * @param marker the marker file
     * @return the trimmed contents, or an empty string when it cannot be read
     */
    private static String readMarker(Path marker) {
        try {
            return Files.readString(marker, StandardCharsets.UTF_8).strip();
        } catch (IOException e) {
            log.warn("Could not read resource marker {}; treating it as stale", marker, e);
            return "";
        }
    }
}
