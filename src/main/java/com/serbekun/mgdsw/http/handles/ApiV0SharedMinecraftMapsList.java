package com.serbekun.mgdsw.http.handles;

import com.serbekun.mgdsw.http.HttpHandler;
import com.serbekun.mgdsw.services.sharedcontent.MinecraftMapsService;

import java.util.List;

import io.javalin.config.JavalinConfig;

public class ApiV0SharedMinecraftMapsList implements HttpHandler {
    

    private final MinecraftMapsService minecraftMapsService;

    public ApiV0SharedMinecraftMapsList(MinecraftMapsService minecraftMapsService) {
        this.minecraftMapsService = minecraftMapsService;
    }

    @Override
    public void register(JavalinConfig config) {
        config.routes.get("/api/v0/shared/minecraft/maps/list", ctx -> {            
            List<String> minecraftMaps = minecraftMapsService.getMinecraftMapsPublicList();
            ctx.json(minecraftMaps);
        });
    }

}
