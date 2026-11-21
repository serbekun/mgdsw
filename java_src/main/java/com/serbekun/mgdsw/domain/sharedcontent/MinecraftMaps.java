package com.serbekun.mgdsw.domain.sharedcontent;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.dataformat.yaml.YAMLMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import com.serbekun.mgdsw.resources.ResourcesBasePath;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.stream.Stream;

/**
 * Access to the Minecraft map metadata stored on disk.
 * <p>
 * This class owns the content-side concerns: it reads map YAML files from the
 * operator-managed content root, keeps the list of available slugs and parses
 * raw YAML bytes into {@link MinecraftMap} objects. It deliberately contains no
 * public API or business rules; those belong to the service layer.
 * </p>
 */
public class MinecraftMaps {

    private static final Logger log = LoggerFactory.getLogger(MinecraftMaps.class);
    private static final String YAML_EXTENSION = ".yaml";

    /** YAML parser for map metadata files. */
    private static final ObjectMapper YAML_MAPPER = YAMLMapper.builder()
            .addModule(new JavaTimeModule())
            .enable(DeserializationFeature.FAIL_ON_NULL_FOR_PRIMITIVES)
            .build();

    // region runtime data

    private final Path minecraftMapsRoot;
    private List<String> minecraftMapsSlugList = List.of();

    // endregion

    /**
     * Creates a map store that reads map YAML from
     * {@code contentRoot/minecraft_maps/}.
     *
     * @param contentRoot the configured content root, e.g. {@code data/content/}
     */
    public MinecraftMaps(Path contentRoot) {
        this.minecraftMapsRoot = contentRoot.resolve(ResourcesBasePath.MINECRAFT_MAPS_YAML.getBasePath());

        buildMinecraftMapsSlugList();
    }

    private void buildMinecraftMapsSlugList() {
        // check is exist minecraftMapsRoot folder and it is not a file.
        if (!Files.isDirectory(minecraftMapsRoot)) {
            log.warn("Minecraft maps directory does not exist: {}", minecraftMapsRoot);
            minecraftMapsSlugList = List.of();
            return;
        }

        try (Stream<Path> files = Files.list(minecraftMapsRoot)) {
            minecraftMapsSlugList = files
                    .filter(Files::isRegularFile)
                    .map(file -> file.getFileName().toString())
                    .filter(name -> name.endsWith(YAML_EXTENSION))
                    .map(MinecraftMaps::stripYamlExtension)
                    .sorted()
                    .toList();
        } catch (IOException e) {
            log.error("Failed to list minecraft maps in {}", minecraftMapsRoot, e);
            minecraftMapsSlugList = List.of();
        }
    }

    public void rebuildMinecraftMapsSlugList() {
        buildMinecraftMapsSlugList();
    }

    private static String stripYamlExtension(String fileName) {
        return fileName.substring(0, fileName.length() - YAML_EXTENSION.length());
    }

    /**
     * All map slugs available in the content root, sorted alphabetically.
     *
     * @return the slugs without the {@code .yaml} extension
     */
    public List<String> getSlugs() {
        return minecraftMapsSlugList;
    }

    /**
     * Reads and parses the map metadata for a slug.
     *
     * @param slug the map slug (the YAML file name without {@code .yaml})
     * @return the parsed map, or {@code null} if the slug is empty, invalid or
     *         the file does not exist
     */
    public MinecraftMap getBySlug(String slug) {
        if (slug == null || slug.isEmpty()) {
            return null;
        }

        String fileName = slug + YAML_EXTENSION;
        try {
            // Validate the slug before touching the filesystem: reject traversal.
            ResourcesBasePath.MINECRAFT_MAPS_YAML.resolve(fileName);
        } catch (IllegalArgumentException e) {
            log.info("Minecraft map: {} has an invalid slug", slug);
            return null;
        }

        Path yamlPath = minecraftMapsRoot.resolve(fileName).normalize();

        if (!yamlPath.startsWith(minecraftMapsRoot) || !Files.isRegularFile(yamlPath)) {
            log.info("Minecraft map: {} not found", slug);
            return null;
        }

        try {
            return parse(Files.readAllBytes(yamlPath));
        } catch (IOException e) {
            log.error("Error parse slug: {}. error message: {}", slug, e);
            return null;
        }
    }

    /**
     * Parses raw YAML bytes into a {@link MinecraftMap}.
     *
     * @param yamlBytes the raw YAML file bytes
     * @return the parsed map
     * @throws IOException if the bytes are not a valid map YAML
     */
    public static MinecraftMap parse(byte[] yamlBytes) throws IOException {
        return YAML_MAPPER.readValue(yamlBytes, MinecraftMap.class);
    }
}
