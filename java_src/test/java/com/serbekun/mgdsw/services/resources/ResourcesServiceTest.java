package com.serbekun.mgdsw.services.resources;

import static com.serbekun.mgdsw.resources.ResourceType.SHARED;
import static com.serbekun.mgdsw.resources.ResourceType.STATIC;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import com.serbekun.mgdsw.resources.ResourcesBasePath;
import com.serbekun.mgdsw.resources.ResourcesCache;

class ResourcesServiceTest {

    @TempDir
    Path temp;

    private Path staticRoot;
    private Path sharedRoot;
    private ResourcesCache cache;
    private ResourcesService service;

    @BeforeEach
    void setUp() throws IOException {
        staticRoot = Files.createDirectories(temp.resolve("static"));
        sharedRoot = Files.createDirectories(temp.resolve("shared"));
        cache = new ResourcesCache(
                Map.of(STATIC, staticRoot, SHARED, sharedRoot), 1024);
        service = new ResourcesService(cache);
    }

    @Test
    void servesStaticResourcesAndCachesThem() throws IOException {
        write(staticRoot, "css/app.css", "body{}");

        assertThat(service.getCss("app.css")).isEqualTo("body{}");
        assertThat(cache.isCached(STATIC, "css/app.css")).isTrue();
    }

    @Test
    void servesSharedResourcesWithoutCachingThem() throws IOException {
        write(sharedRoot, "minecraft_maps_zips/map.zip", new byte[] {1, 2, 3});

        ResourcesService.ResourceData data =
                service.getResource(ResourcesBasePath.MINECRAFT_MAPS_ZIPS, "map.zip");

        assertThat(data).isNotNull();
        assertThat(data.data()).containsExactly(1, 2, 3);
        assertThat(data.etag()).isNull();
        assertThat(data.contentType()).isEqualTo("application/zip");
        assertThat(cache.isCached(SHARED, "minecraft_maps_zips/map.zip")).isFalse();
    }

    @Test
    void neverServesContentResources() {
        assertThatThrownBy(() ->
                service.getResource(ResourcesBasePath.MINECRAFT_MAPS_YAML, "map.yaml"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("CONTENT");

        assertThatThrownBy(() ->
                service.getResource(ResourcesBasePath.MINECRAFT_MAPS_YAML, ""))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("CONTENT");

        assertThatThrownBy(() ->
                service.listResources(ResourcesBasePath.MINECRAFT_MAPS_YAML))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("CONTENT");

        assertThatThrownBy(() ->
                service.listAsJson(ResourcesBasePath.MINECRAFT_MAPS_YAML))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("CONTENT");
    }

    private static void write(Path root, String relative, String content) throws IOException {
        write(root, relative, content.getBytes(UTF_8));
    }

    private static void write(Path root, String relative, byte[] content) throws IOException {
        Path file = root.resolve(relative);
        Files.createDirectories(file.getParent());
        Files.write(file, content);
    }
}
