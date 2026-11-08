package com.serbekun.mgdsw.config.configs;

public record ConfigResources(long maxCacheableSize,
    String staticBasePath,
    String contentBasePath,
    String sharedBasePath,
    String examplesBasePath
) {
    public ConfigResources {
        if (maxCacheableSize < 0) {
            throw new IllegalArgumentException("maxCacheableSize must not be negative");
        }
    }
}