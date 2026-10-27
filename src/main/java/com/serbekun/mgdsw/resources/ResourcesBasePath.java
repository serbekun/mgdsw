package com.serbekun.mgdsw.resources;

import java.util.Arrays;
import java.util.List;

import com.serbekun.mgdsw.services.resources.ResourcesService;

/**
 * Single source of truth for static resource directory paths.
 * <p>
 * Every enum constant represents a resource directory (HTML, CSS, JS, images,
 * partials, Minecraft map zips) together with its base path inside the
 * application resources. It is the single point where all resource directory
 * names are defined, so {@link ResourcesService}, {@link ResourcesLoader} and
 * the first-run unpacker all agree on where each kind of resource lives.
 * </p>
 * <p>
 * Instances additionally validate resource names passed to {@link #resolve}
 * to reject path traversal attempts, so callers can never address a file
 * outside its own resource directory.
 * </p>
 */
public enum ResourcesBasePath {

    /** Frontend static files (templates that exist inside the JAR). */
    HTML("html/", UnpackPolicy.PACKAGED, CachePolicy.CACHE, ResourceType.STATIC),

    /** Stylesheets packaged with the application. */
    CSS("css/", UnpackPolicy.PACKAGED, CachePolicy.CACHE, ResourceType.STATIC),

    /** JavaScript files packaged with the application. */
    JS("js/", UnpackPolicy.PACKAGED, CachePolicy.CACHE, ResourceType.STATIC),

    /** Image assets packaged with the application. */
    IMAGES("images/", UnpackPolicy.PACKAGED, CachePolicy.CACHE, ResourceType.STATIC),

    /** HTML template partials packaged with the application. */
    PARTIALS("html/partials/", UnpackPolicy.PACKAGED, CachePolicy.CACHE, ResourceType.STATIC),

    /** Minecraft map ZIP archives shared from the Maxim Games Discord server. */
    MINECRAFT_MAPS_ZIPS("minecraft_maps_zips/", UnpackPolicy.DISK_ONLY, CachePolicy.DISABLE_CACHE,  ResourceType.SHARED),

    /** Minecraft map YAML where will be saved map metadata. */
    MINECRAFT_MAPS_YAML("minecraft_maps", UnpackPolicy.DISK_ONLY, CachePolicy.DISABLE_CACHE, ResourceType.CONTENT);

    private final String basePath;
    private final UnpackPolicy unpackPolicy;
    private final CachePolicy cachePolicy;
    private final ResourceType resourceType;

    /**
     * Creates a new resource base path entry.
     *
     * @param basePath the directory path inside resources; a trailing slash is
     *                 added automatically if missing
     * @param policy   whether the directory's contents are packaged in the JAR
     *                 or exist only on disk
     */
    ResourcesBasePath(String basePath, UnpackPolicy UnpackPolicy, CachePolicy cachePolicy, ResourceType resourceType) {
        this.basePath = basePath.endsWith("/") ? basePath : basePath + "/";
        this.unpackPolicy = UnpackPolicy;
        this.cachePolicy = cachePolicy;
        this.resourceType = resourceType;
    }

    /** The directory path this entry represents, always ending with {@code '/'}.
     *
     * @return the base path
     */
    public String getBasePath() {
        return basePath;
    }

    /**
     * Whether this directory's resources are packaged in the JAR.
     *
     * @return {@code true} if packaged, {@code false} if disk-only
     */
    public boolean isPackaged() {
        return unpackPolicy == UnpackPolicy.PACKAGED;
    }

    /**
     * The cache policy for this directory's resources.
     *
     * @return the cache policy
     */
    public CachePolicy getCachePolicy() {
        return cachePolicy;
    }

    public ResourceType getResourceType() {
        return resourceType;
    }

    /**
     * Whether this directory's resources are cached in the {@link ResourcesCache}.
     *
     * @return {@code true} if cached, {@code false} if cache is disabled
     */
    public boolean isCached() {
        return cachePolicy == CachePolicy.CACHE;
    }

    /**
     * All directories that must exist on disk (for empty forks).
     *
     * @return every {@link ResourcesBasePath} value
     */
    public static List<ResourcesBasePath> directoriesToCreate() {
        return List.of(values());
    }

    /**
     * Only those directories whose templates must be extracted from the JAR.
     *
     * @return the packaged subset of {@link ResourcesBasePath} values
     */
    public static List<ResourcesBasePath> packagedDirectories() {
        return Arrays.stream(values())
                .filter(ResourcesBasePath::isPackaged)
                .toList();
    }

    /**
     * Resolves the full path for a resource of this type.
     *
     * <p>The supplied name is validated before being appended, so path
     * traversal sequences ({@code ..}, {@code /}, {@code \}, {@code %}) cause
     * an {@link IllegalArgumentException} and the caller can never address a
     * file outside its own resource directory.</p>
     *
     * @param name the resource file name inside this directory
     * @return the resolved full path ({@code basePath + name})
     * @throws IllegalArgumentException if the name contains path traversal
     *                                  characters
     */
    public String resolve(String name) {
        if (name == null) {
            name = "";
        }
        if (name.contains("..") || name.contains("/") || name.contains("\\") || name.contains("%")) {
            throw new IllegalArgumentException("Invalid resource name: path traversal not allowed");
        }
        return basePath + name;
    }
}
