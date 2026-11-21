package com.serbekun.mgdsw;

import java.util.regex.Pattern;

/**
 * Single registry ("hub") for every environment variable name used by mgdsw.
 *
 * <p>The purpose of this enum is to avoid scattering hard-coded environment
 * variable names as string literals across the codebase (for example
 * {@code System.getenv("MGDSW_CONFIG")}). Each environment variable is declared
 * exactly once as an enum constant, and the rest of the application refers to
 * that constant and reads its value through {@link #value()}.
 *
 * <p>This provides:
 * <ul>
 *   <li>one place to discover all environment variables the application supports;</li>
 *   <li>compile-time safety and refactoring support instead of raw strings;</li>
 *   <li>a single place for name validation and environment lookup logic.</li>
 * </ul>
 *
 * <p>Typical usage:
 * <pre>{@code
 *   String configPath = Envs.MGDSW_CONFIG.value();
 *   if (configPath == null) {
 *       config = configLoader.loadDefault();
 *   } else {
 *       config = configLoader.load(Path.of(configPath));
 *   }
 * }</pre>
 *
 * <p>Every constant is validated in the constructor: a malformed name fails
 * fast with {@link IllegalArgumentException} during class initialization.
 *
 * <p>TODO: decide whether values should be cached at startup or read on every
 * call; caching would make the configuration immutable for the process lifetime.
 * <p>TODO: add typed accessors (for example {@code path()} or {@code intValue()})
 * once non-string environment variables appear.
 * <p>TODO: allow overriding values (for example via system properties) to make
 * code that depends on environment variables easier to test.
 */
public enum Envs {

    /**
     * Path to the YAML configuration file.
     *
     * <p>When the variable is not set (or empty), {@code ConfigLoader} falls
     * back to the bundled classpath config {@code config/config.default.yaml}.
     */
    MGDSW_CONFIG("MGDSW_CONFIG");

    /**
     * Holder for the name pattern.
     *
     * <p>The pattern cannot be a plain static field of {@link Envs}: enum
     * constants are initialized before static fields declared after them, so
     * the constructor would see a {@code null} pattern during class
     * initialization. A nested holder is initialized lazily on first use,
     * independently of enum constant order.
     */
    private static final class EnvNamePattern {
        /** Valid environment variable name: a letter or underscore followed by letters, digits or underscores. */
        private static final Pattern INSTANCE =
                Pattern.compile("[A-Za-z_][A-Za-z0-9_]*");
    }

    /** Raw name of the environment variable as seen by the operating system. */
    private final String name;

    /**
     * Creates a registry entry.
     *
     * @param name raw environment variable name
     * @throws IllegalArgumentException if {@code name} is {@code null} or is not
     *                                  a valid environment variable name
     */
    Envs(String name) {
        if (!isValidEnvName(name)) {
            throw new IllegalArgumentException(
                    "Invalid environment variable name: \"" + name + "\"");
        }
        this.name = name;
    }

    /**
     * Returns the raw environment variable name.
     *
     * <p>Use this only where a name (not a value) is required, for example in log
     * or error messages. Normal consumers should call {@link #value()}.
     *
     * @return environment variable name, never {@code null}
     */
    public String envName() {
        return name;
    }

    /**
     * Reads the current value of this environment variable from the process
     * environment.
     *
     * @return the variable value, or {@code null} if it is not set
     * @see System#getenv(String)
     */
    public String value() {
        return System.getenv(name);
    }

    /**
     * Checks whether this environment variable is currently set.
     *
     * @return {@code true} if {@link #value()} is not {@code null}
     */
    public boolean isSet() {
        return value() != null;
    }

    /**
     * Checks that the given string is a syntactically valid environment variable
     * name.
     *
     * <p>The rule is intentionally permissive: any name accepted by common Unix
     * shells and by {@link System#getenv(String)} matches, i.e. it starts with a
     * letter or underscore and continues with letters, digits or underscores.
     *
     * @param name candidate name, may be {@code null}
     * @return {@code true} if the name is valid, {@code false} otherwise
     */
    public static boolean isValidEnvName(String name) {
        return name != null && EnvNamePattern.INSTANCE.matcher(name).matches();
    }
}
