package com.serbekun.mgdsw.http;

import io.javalin.config.JavalinConfig;

/**
 * Contract for self-registering endpoint handlers.
 * Each implementation registers its own routes (and its own auth endpoints)
 * directly on the {@link JavalinConfig} passed in, before the server is
 * started: the server self-initializes all endpoints from these registrations
 * when {@code Javalin.create(config -&gt; ...)} runs, so no handler ever sees a
 * running server. Handlers know nothing about each other; they receive only
 * their own dependencies through the constructor.
 */
public interface HttpHandler {
    void register(JavalinConfig config);
}
