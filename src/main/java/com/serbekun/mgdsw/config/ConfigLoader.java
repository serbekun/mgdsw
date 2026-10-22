package com.serbekun.mgdsw.config;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory;
import com.serbekun.mgdsw.Main;
import com.serbekun.mgdsw.config.configs.ConfigHttpInit;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Reads, validates and assembles the application configuration.
 *
 * <p>The loader resolves a YAML file in two steps:
 * <ol>
 *   <li>First it tries the path given to {@link #load(Path)} on disk.</li>
 *   <li>If the file does not exist there, it falls back to the bundled
 *       classpath resource {@value #DEFAULT_RESOURCE}.</li>
 * </ol>
 *
 * <p>The YAML is parsed with Jackson into the intermediate {@link RawConfig}
 * record, then converted into an immutable {@link ConfigContext} which is the
 * single object handed to the rest of the application (e.g. from {@code main}).
 *
 * <p>Deserialization is strict: {@link DeserializationFeature#FAIL_ON_MISSING_CREATOR_PROPERTIES}
 * is enabled, so a missing {@code http} section or a missing {@code http.port}
 * fails fast instead of silently defaulting (for example to port {@code 0}).
 *
 * <p>Typical usage:
 * <pre>{@code
 *   ConfigContext config = new ConfigLoader().load(Path.of("config.yaml"));
 * }</pre>
 *
 * @see ConfigContext
 * @see ConfigHttpInit
 */
public class ConfigLoader {

    private static final Logger log = LoggerFactory.getLogger(ConfigLoader.class);


    /** Fallback config inside the classpath ({@code src/main/resources/config/config.default.yaml}). */
    private static final String DEFAULT_RESOURCE = "config/config.default.yaml";

    private static final ObjectMapper MAPPER = new ObjectMapper(new YAMLFactory())
            // A missing field is an error, not a silent port = 0
            .enable(DeserializationFeature.FAIL_ON_MISSING_CREATOR_PROPERTIES);

    /** Structure of the YAML file. Section keys map to the names of the record components. */
    record RawConfig(ConfigHttpInit http) {}

    /**
     * Loads the config: first from disk at the given path, and if the file is
     * absent there, from the classpath ({@code config/config.default.yaml}).
     *
     * @param path path to the YAML file on disk (absolute or relative to the working directory)
     * @return loaded and validated configuration context
     * @throws IllegalStateException if the file on disk exists but cannot be read or is invalid,
     *                               or if neither a disk file nor a classpath resource is present
     */
    public ConfigContext load(Path path) {
        if (path == null) {
            log.info("ENV \"MGDSW_CONFIG\" not exported using default");
            return loadDefault();
        }
        
        if (Files.exists(path)) {
            try (InputStream in = Files.newInputStream(path)) {
                return parse(in, path.toString());
            } catch (IOException e) {
                throw new IllegalStateException("Cannot load config " + path + ": " + e.getMessage(), e);
            }
        }
        return loadDefault();
    }

    /** Loads the fallback config from the classpath. */
    public ConfigContext loadDefault() {
        try (InputStream in = ConfigLoader.class.getClassLoader().getResourceAsStream(DEFAULT_RESOURCE)) {
            if (in == null) {
                throw new IllegalStateException(
                        "Config file not found on disk and no default resource '" + DEFAULT_RESOURCE + "' in classpath");
            }
            return parse(in, "classpath:" + DEFAULT_RESOURCE);
        } catch (IOException e) {
            throw new IllegalStateException("Cannot load default config " + DEFAULT_RESOURCE + ": " + e.getMessage(), e);
        }
    }

    /** Parses the YAML stream into a {@link ConfigContext}, validating required sections. */
    private ConfigContext parse(InputStream in, String source) throws IOException {
        RawConfig raw = MAPPER.readValue(in, RawConfig.class);
        if (raw == null || raw.http() == null) {
            throw new IllegalStateException("Section 'http' is missing in " + source);
        }
        return new ConfigContext(raw.http());
    }
}