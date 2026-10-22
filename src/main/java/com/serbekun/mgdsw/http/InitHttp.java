package com.serbekun.mgdsw.http;

import java.util.List;

import com.serbekun.mgdsw.config.configs.ConfigHttpInit;
import com.serbekun.mgdsw.http.handles.ApiV0Health;

import io.javalin.Javalin;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Bootstrap and lifecycle manager for the embedded Javalin HTTP server.
 *
 * <p>This class is the single entry point for starting the HTTP layer of the
 * application. It collects all {@link HttpHandler} implementations into one
 * list, registers them on a {@link Javalin} instance via its configuration
 * phase, and starts the server on a dedicated background thread
 * ({@code http-server-thread}) so that application startup is not blocked.
 *
 * <p>A JVM shutdown hook is installed so that pressing {@code Ctrl+C} (or any
 * other JVM termination) stops the server gracefully.
 *
 * <p>Typical usage:
 * <pre>{@code
 *   InitHttp http = new InitHttp();
 *   http.init();
 * }</pre>
 */
public class InitHttp {

    private final ConfigHttpInit configHttpInit;

    /** Logger for this class. */
    private static final Logger log = LoggerFactory.getLogger(InitHttp.class);

    /** Self-registering endpoint handlers that are applied to the server config on startup. */
    private List<HttpHandler> handlers;

    /** The embedded Javalin server instance; assigned once the server thread starts. */
    private Javalin server;

    /** Creates a new {@code InitHttp} instance. Does not start the server; call {@link #init()} for that. */
    public InitHttp(ConfigHttpInit configHttpInit) {
        this.configHttpInit = configHttpInit;
    }

    /**
     * Initializes the handler list and starts the embedded Javalin HTTP server
     * on a dedicated background thread.
     *
     * <p>Steps performed:
     * <ol>
     *   <li>Builds the list of {@link HttpHandler}s (e.g. {@link ApiV0Health}).</li>
     *   <li>Installs a shutdown hook for graceful server stop.</li>
     *   <li>Creates and starts a {@code http-server-thread} that creates the
     *       Javalin instance, registers all handlers, and binds to port
     *       {@value #HTTP_PORT}.</li>
     * </ol>
     */
    public void init() {

        log.info("Creating handlers list");
        handlers = List.of(
            new ApiV0Health()
        );

        addShutdownHook(server);

        Thread httpServerThread = new Thread(() -> {
            server = Javalin.create(config -> {
                handlers.forEach(handler -> handler.register(config));
            }).start(configHttpInit.port());
        }, "http-server-thread");

        log.info("Start http server thread");
        httpServerThread.start();
    }

    /**
     * Registers a JVM shutdown hook that gracefully stops the Javalin server
     * when the JVM exits (e.g. {@code Ctrl+C}).
     *
     * @param server the Javalin instance to stop on shutdown; may be {@code null}
     *               at hook-registration time and is captured by the hook thread
     */
    private static void addShutdownHook(Javalin server) {
        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            log.info("Shutting down server...");
            server.stop();
        }, "shutdown-hook"));
    }
}
