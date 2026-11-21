package com.serbekun.mgdsw.http.handles;

import com.serbekun.mgdsw.domain.http.ApiV0SharedMinecraftMapsYamlRes;
import com.serbekun.mgdsw.domain.http.ApiV0SharedMinecraftMapsYamlResNotFound;
import com.serbekun.mgdsw.http.HttpHandler;
import com.serbekun.mgdsw.services.sharedcontent.MinecraftMapsService;

import io.javalin.config.JavalinConfig;

/**
 * Public Minecraft map metadata endpoint:
 * {@code GET /api/v0/shared/minecraft/maps/{slug}/yaml}.
 *
 * <p>Returns the public metadata of a single published Minecraft map as JSON
 * with snake_case field names: {@code slug}, {@code name}, {@code authors},
 * {@code description}, {@code license}, {@code map_version},
 * {@code release_date}, {@code minecraft_version}, {@code preview} and
 * {@code zip_sha256}. The internal {@code published} flag is intentionally not
 * included. If the map does not exist or is not published, the endpoint
 * responds with {@code 404 Not Found}.
 *
 * <p>Needed so a site or client can display a map page, install the map and
 * verify the integrity of the downloaded zip using {@code zip_sha256}, without
 * revealing which maps exist but are not published yet.
 *
 * <p>Request: no body; the map is selected by the {@code slug} path parameter.
 *
 * <p>Response: {@code 200 OK}, {@code Content-Type: application/json}
 * <pre>{@code
 * {
 *   "slug": "example-map",
 *   "name": "Example Map",
 *   "authors": ["Author One", "Author Two"],
 *   "description": "A short description of the map.",
 *   "license": "CC BY-SA 4.0",
 *   "map_version": "1.0.0",
 *   "release_date": "2026-01-01",
 *   "minecraft_version": "1.21.4",
 *   "preview": false,
 *   "zip_sha256": "0000...0000"
 * }
 * }</pre>
 *
 * <p>Response for an unknown or unpublished map: {@code 404 Not Found},
 * {@code Content-Type: application/json}
 * <pre>{@code
 * {"error": "slug: example-map. not found"}
 * }</pre>
 */
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