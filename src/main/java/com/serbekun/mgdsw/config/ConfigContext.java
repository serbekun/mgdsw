package com.serbekun.mgdsw.config;

import com.serbekun.mgdsw.config.configs.ConfigHttpInit;


/**
 * Aggregate holder for every configuration section of the application.
 *
 * <p>{@link ConfigLoader} is responsible for reading, validating and assembling
 * the raw configuration sources; this class is the single object it hands back.
 * Instead of passing each individual config record through the codebase, a
 * loaded {@code ConfigContext} is passed wherever configuration is needed.
 *
 * <p>Typical usage:
 * <pre>{@code
 *   Path configFilePath = Path.of(System.getenv("MGDSW_CONFIG"));
 *   ConfigContext config = new ConfigLoader().load(configFilePath);
 *
 *   InitHttp http = new InitHttp(config.http());
 *   http.init();
 * }</pre>
 *
 * <p>Instances are immutable value containers: the record components are
 * {@code final} and every referenced config type is itself a record, so the
 * context can safely be shared across threads.
 *
 * @see ConfigLoader
 * @see ConfigHttpInit
 */
public record ConfigContext(ConfigHttpInit http) {
}
