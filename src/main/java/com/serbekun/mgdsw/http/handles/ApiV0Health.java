package com.serbekun.mgdsw.http.handles;

import com.serbekun.mgdsw.domain.http.ApiV0HealthRes;
import com.serbekun.mgdsw.http.HttpHandler;
import io.javalin.config.JavalinConfig;

/**
 * Health check endpoint: {@code GET /api/v0/health}.
 *
 * <p>Reports that the HTTP server is up and able to answer requests. The
 * handler has no dependencies and always responds with HTTP {@code 200} and a
 * JSON body whose {@code status} field is {@code "ok"}.
 *
 * <p>Needed by monitoring, load balancers and deployment smoke tests to verify
 * that the process is alive without touching application data or external
 * services.
 *
 * <p>Request: no path or query parameters and no body.
 *
 * <p>Response: {@code 200 OK}, {@code Content-Type: application/json}
 * <pre>{@code
 * {"status": "ok"}
 * }</pre>
 */
public final class ApiV0Health implements HttpHandler {

    @Override
    public void register(JavalinConfig config) {
        config.routes.get("/api/v0/health", ctx ->
            ctx.json(new ApiV0HealthRes("ok"))
        );
    }
}