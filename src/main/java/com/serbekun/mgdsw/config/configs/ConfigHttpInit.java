package com.serbekun.mgdsw.config.configs;

/**
 * Configuration for the embedded HTTP server.
 *
 * <p>Maps the {@code http} section of the YAML config. The {@code port} is
 * mandatory (missing values fail during deserialization in
 * {@link com.serbekun.mgdsw.config.ConfigLoader}) and must be a valid TCP port
 * in the range {@code 1..65535}; any other value is rejected with an
 * {@link IllegalArgumentException}.
 *
 * @param port TCP port the HTTP server binds to
 */
public record ConfigHttpInit(int port) {
    public ConfigHttpInit {
        if (port < 1 || port > 65535) {
            throw new IllegalArgumentException("port out of range: " + port);
        }
    }
}