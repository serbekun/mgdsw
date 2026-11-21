package com.serbekun.mgdsw.resources;

import static com.serbekun.mgdsw.resources.ResourceType.CONTENT;
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

class ResourcesCacheTest {

    @TempDir
    Path temp;

    private Path staticRoot;
    private Path sharedRoot;
    private ResourcesCache cache;

    @BeforeEach
    void setUp() throws IOException {
        staticRoot = Files.createDirectories(temp.resolve("static"));
        sharedRoot = Files.createDirectories(temp.resolve("shared"));
        cache = new ResourcesCache(
                Map.of(STATIC, staticRoot, SHARED, sharedRoot), 1024);
    }

    @Test
    void cachesStaticResources() throws IOException {
        write(staticRoot, "css/app.css", "body{}");

        ResourcesCache.Entry entry = cache.get(STATIC, "css/app.css", CachePolicy.CACHE);

        assertThat(entry).isNotNull();
        assertThat(entry.data()).isEqualTo("body{}".getBytes(UTF_8));
        assertThat(entry.etag()).isNotBlank();
        assertThat(cache.isCached(STATIC, "css/app.css")).isTrue();
    }

    @Test
    void cachesStaticMisses() {
        assertThat(cache.get(STATIC, "css/missing.css", CachePolicy.CACHE)).isNull();
        assertThat(cache.isCached(STATIC, "css/missing.css")).isTrue();
    }

    @Test
    void neverCachesSharedEvenWhenPolicyAsksForIt() throws IOException {
        Path zip = write(sharedRoot, "minecraft_maps_zips/map.zip", new byte[] {1, 2, 3});

        ResourcesCache.Entry first = cache.get(SHARED, "minecraft_maps_zips/map.zip", CachePolicy.CACHE);

        assertThat(first.data()).containsExactly(1, 2, 3);
        assertThat(first.etag()).isNull();
        assertThat(cache.isCached(SHARED, "minecraft_maps_zips/map.zip")).isFalse();

        // A second lookup must see the new bytes: the file was not retained.
        Files.write(zip, new byte[] {4, 5});
        ResourcesCache.Entry second = cache.get(SHARED, "minecraft_maps_zips/map.zip", CachePolicy.CACHE);

        assertThat(second.data()).containsExactly(4, 5);
    }

    @Test
    void rejectsContentResources() {
        assertThatThrownBy(() -> cache.get(CONTENT, "minecraft_maps/map.yaml", CachePolicy.CACHE))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("CONTENT");

        assertThatThrownBy(() -> cache.listResources(CONTENT, "minecraft_maps"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("CONTENT");
    }

    @Test
    void keepsEqualPathsFromDifferentRootsApart() throws IOException {
        write(staticRoot, "same/file.txt", "static");
        write(sharedRoot, "same/file.txt", "shared");

        assertThat(new String(cache.get(STATIC, "same/file.txt", CachePolicy.CACHE).data(), UTF_8))
                .isEqualTo("static");
        assertThat(new String(cache.get(SHARED, "same/file.txt", CachePolicy.CACHE).data(), UTF_8))
                .isEqualTo("shared");
    }

    private static Path write(Path root, String relative, String content) throws IOException {
        return write(root, relative, content.getBytes(UTF_8));
    }

    private static Path write(Path root, String relative, byte[] content) throws IOException {
        Path file = root.resolve(relative);
        Files.createDirectories(file.getParent());
        Files.write(file, content);
        return file;
    }
}
