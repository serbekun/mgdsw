package com.serbekun.mgdsw.config.configs;

public record ConfigResources(long maxCacheableSize) {
    public ConfigResources {
        if (maxCacheableSize < 0) {
            throw new IllegalArgumentException("maxCacheableSize must not be negative");
        }
    }
}