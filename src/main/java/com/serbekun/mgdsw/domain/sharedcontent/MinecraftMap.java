package com.serbekun.mgdsw.domain.sharedcontent;

import java.time.LocalDate;
import java.util.List;

public record MinecraftMap(
    String slug, // value that will be used for format url for download
    String name, // Map name (showed on page)
    List<String> authors, // Map authors
    String description, // Map description
    String license, // Map license
    String mapVersion, // Minecraft Map version.
    LocalDate releaseDate, // Date when map was released.
    String minecraftVersion, // Minecraft version under which map was created.
    boolean published, // If true you can see map on site and download zip.
    boolean preview, // If true you can see map on site but you can't download zip.
    String zipSha256 // 64 hex go to header.
) {}
