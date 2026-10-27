package com.serbekun.mgdsw.resources;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.nio.file.Files;
import java.nio.file.Path;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import com.serbekun.mgdsw.config.configs.ConfigResources;

class ResourcesUnpackerTest {

    private static final String VERSION = "test-version";

    @TempDir
    Path temp;

    @Test
    void createsEveryRootAndUnpacksTemplatesIntoStaticRoot() throws Exception {
        Path staticRoot = temp.resolve("static");
        Path contentRoot = temp.resolve("content");
        Path sharedRoot = temp.resolve("shared");

        ResourcesUnpacker unpacker = unpackerFor(staticRoot);

        assertThat(unpacker.unpack(config(staticRoot, contentRoot, sharedRoot))).isTrue();

        assertThat(staticRoot.resolve("html").resolve("resource_test.html")).isRegularFile();
        assertThat(contentRoot.resolve("minecraft_maps")).isDirectory();
        assertThat(sharedRoot.resolve("minecraft_maps_zips")).isDirectory();
        assertThat(Files.readString(staticRoot.resolve(ResourcesUnpacker.MARKER_FILE)).strip())
                .isEqualTo(VERSION);

        try (var entries = Files.list(contentRoot.resolve("minecraft_maps"))) {
            assertThat(entries.toList()).isEmpty();
        }
    }

    @Test
    void keepsExistingFilesAndSkipsCopyWhenVersionMatches() throws Exception {
        Path staticRoot = temp.resolve("static");
        Path contentRoot = temp.resolve("content");
        Path sharedRoot = temp.resolve("shared");
        ConfigResources config = config(staticRoot, contentRoot, sharedRoot);

        ResourcesUnpacker unpacker = unpackerFor(staticRoot);
        assertThat(unpacker.unpack(config)).isTrue();

        Path template = staticRoot.resolve("html").resolve("resource_test.html");
        Files.writeString(template, "edited by the operator");

        assertThat(unpacker.unpack(config)).isFalse();
        assertThat(template).hasContent("edited by the operator");
    }

    @Test
    void rejectsRootsThatAreNestedInsideEachOther() {
        Path staticRoot = temp.resolve("static");
        Path sharedRoot = temp.resolve("shared");

        ResourcesUnpacker unpacker = unpackerFor(staticRoot);

        assertThatThrownBy(() -> unpacker.unpack(
                config(staticRoot, staticRoot.resolve("content"), sharedRoot)))
                .isInstanceOf(IllegalStateException.class)
                .hasMessageContaining("separate");
    }

    @Test
    void rejectsBlankRoot() {
        Path staticRoot = temp.resolve("static");
        Path sharedRoot = temp.resolve("shared");

        ResourcesUnpacker unpacker = unpackerFor(staticRoot);

        assertThatThrownBy(() -> unpacker.unpack(
                new ConfigResources(1024, " ", temp.resolve("content").toString(), sharedRoot.toString())))
                .isInstanceOf(IllegalStateException.class)
                .hasMessageContaining("staticBasePath");
    }

    private ResourcesUnpacker unpackerFor(Path staticRoot) {
        return new ResourcesUnpacker(new ResourcesLoader(staticRoot), VERSION);
    }

    private ConfigResources config(Path staticRoot, Path contentRoot, Path sharedRoot) {
        return new ConfigResources(1024, staticRoot.toString(), contentRoot.toString(), sharedRoot.toString());
    }
}
