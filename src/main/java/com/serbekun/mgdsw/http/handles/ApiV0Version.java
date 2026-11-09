package com.serbekun.mgdsw.http.handles;

import com.serbekun.mgdsw.BuildInfo;
import com.serbekun.mgdsw.http.HttpHandler;

import io.javalin.config.JavalinConfig;

import com.serbekun.mgdsw.domain.http.ApiV0VersionRes;

public class ApiV0Version implements HttpHandler {
    
    @Override 
    public void register(JavalinConfig config) {
        config.routes.get("/api/v0/version", ctx -> {
            ctx.json(new ApiV0VersionRes(BuildInfo.version()));
        });
    }
}
