package com.serbekun.mgdsw.http.handles;

import com.serbekun.mgdsw.domain.http.ApiV0SharedMinecraftMapsYamlRes;
import com.serbekun.mgdsw.domain.http.ApiV0SharedMinecraftMapsYamlResNotFound;
import com.serbekun.mgdsw.http.HttpHandler;
import com.serbekun.mgdsw.services.sharedcontent.MinecraftMapsService;

import io.javalin.config.JavalinConfig;

public final class ApiV0SharedMinecraftMapsYaml implements HttpHandler {

    private final MinecraftMapsService minecraftMapsService;

    public ApiV0SharedMinecraftMapsYaml(MinecraftMapsService minecraftMapsService) {
        this.minecraftMapsService = minecraftMapsService;
    }

    @Override
    public void register(JavalinConfig config) {
        config.routes.get("/api/v0/shared/minecraft/maps/{slug}/yaml", ctx -> {
            String slug = ctx.pathParam("slug");

            ApiV0SharedMinecraftMapsYamlRes response =
                minecraftMapsService.getMinecraftMapPublic(slug);

            if (response == null) {
                ctx.status(404);
                ctx.json(new ApiV0SharedMinecraftMapsYamlResNotFound(
                    "slug: %s. not found".formatted(slug)));
                return;
            }

            ctx.json(response);
        });
    }
}