package com.serbekun.mgdsw.http;

import io.javalin.Javalin;

/**
 * Contract for self-registering endpoint handlers.
 * Each implementation registers its own routes (and its own auth endpoints)
 * on the given server. Handlers know nothing about each other; they receive
 * only their own dependencies through the constructor.
 */
public interface HttpHandler {
    void register(Javalin server);
}