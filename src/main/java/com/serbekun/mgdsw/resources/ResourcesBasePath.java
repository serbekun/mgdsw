package com.serbekun.mgdsw.resources;

public enum ResourcesBasePath {

    HTML("html/"),
    CSS("css/"),
    JS("js/"),
    IMAGES("images/"),
    VIDEO("video/"),
    PDF("pdf/"),
    PARTIALS("html/partials/");

    private final String basePath;

    ResourcesBasePath(String basePath) {
        this.basePath = basePath.endsWith("/") ? basePath : basePath + "/";
    }

    public String getBasePath() {
        return basePath;
    }

    /**
     * Resolves the full path for a resource of this type.
     * Rejects names containing path traversal sequences.
     */
    public String resolve(String name) {
        if (name == null) {
            name = "";
        }
        if (name.contains("..") || name.contains("/") || name.contains("\\") || name.contains("%")) {
            throw new IllegalArgumentException("Invalid resource name: path traversal not allowed");
        }
        return basePath + name;
    }
}