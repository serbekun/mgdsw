package com.serbekun.mgdsw.http.handles;

import com.serbekun.mgdsw.http.HttpHandler;
import com.serbekun.mgdsw.services.sharedcontent.MinecraftMapsService;

import java.util.List;

import io.javalin.config.JavalinConfig;

/**
 * Public Minecraft maps list endpoint:
 * {@code GET /api/v0/shared/minecraft/maps/list}.
 *
 * <p>Returns a JSON array of the slugs of all <b>published</b> Minecraft maps.
 * The list comes from a cached snapshot built by
 * {@link MinecraftMapsService} at startup and refreshed via
 * {@code rebuildMinecraftMapsSlugList()}, so unpublished maps are never
 * exposed to outside callers.
 *
 * <p>Needed as the public catalogue of available maps: a site or client uses
 * the returned slugs to then request the YAML metadata of a specific map.
 *
 * <p>Request: no path or query parameters and no body.
 *
 * <p>Response: {@code 200 OK}, {@code Content-Type: application/json}; an
 * array that may be empty
 * <pre>{@code
 * ["example-map", "another-map"]
 * }</pre>
 */
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
