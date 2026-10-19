package com.serbekun.mgdsw.http.handles;

import com.serbekun.mgdsw.domain.http.ApiV0HealthRes;
import com.serbekun.mgdsw.http.HttpHandler;
import io.javalin.config.JavalinConfig;

public final class ApiV0Health implements HttpHandler {

    @Override
    public void register(JavalinConfig config) {
        config.routes.get("/api/v0/health", ctx ->
            ctx.json(new ApiV0HealthRes("ok"))
        );
    }
}