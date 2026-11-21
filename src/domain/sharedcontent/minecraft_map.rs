use chrono::NaiveDate;

pub struct MinecraftMap {
    slug: String,              // value that will be used for format url for download
    name: String,              // Map name (showed on page)
    authors: Vec<String>,      // Map authors
    description: String,       // Map description
    license: String,           // Map license
    map_version: String,       // Minecraft Map version.
    release_date: NaiveDate,   // Date when map was released.
    minecraft_version: String, // Minecraft version under which map was created.
    published: bool,           // If true you can see map on site and download zip.
    preview: bool,             // If true you can see map on site but you can't download zip.
    zip_sha256: String,        // 64 hex go to header.
}
