package com.serbekun.mgdsw.domain.http;

import java.time.LocalDate;
import java.util.List;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

/**
 * Copy of {@link com.serbekun.mgdsw.domain.sharedcontent.MinecraftMap} but without
 * `published` field because that field need only for the server that decide give zip/yaml or no.
 */
@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record ApiV0SharedMinecraftMapsYamlRes(
    String slug, // value that will be used for format url for download
    String name, // Map name (showed on page)
    List<String> authors, // Map authors
    String description, // Map description
    String license, // Map license
    String mapVersion, // Minecraft Map version.
    LocalDate releaseDate, // Date when map was released.
    String minecraftVersion, // Minecraft version under which map was created.
    boolean preview, // If true you can see map on site but you can't download zip.
    String zipSha256 // 64 hex go to header.
) {}
