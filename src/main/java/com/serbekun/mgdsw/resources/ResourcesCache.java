package com.serbekun.mgdsw.resources;

import java.nio.charset.StandardCharsets;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;
import java.util.List;

/**
 * Thread-safe cache layer for resources.
 * <p>
 * Only bytes are cached — the server answers with bytes, so keeping a decoded
 * copy would both duplicate memory and re-encode on every response. Text is
 * decoded on demand, always as UTF-8.
 * </p>
 * <p>
 * Misses are cached too: a bot scanning for files that do not exist must not
 * reach disk and classloader on every request.
 * </p>
 * <p>
 * Callers can override the cache policy per lookup. Resources whose directory
 * policy is {@link CachePolicy#DISABLE_CACHE} are loaded fresh on every access
 * and are never stored. Independently of the policy, resources larger than
 * {@link #DEFAULT_MAX_CACHEABLE_SIZE} are returned but not cached, so a large
 * archive cannot retain its bytes in memory forever.
 * </p>
 */
public class ResourcesCache {

    /** Default upper bound (100 MiB) for a resource to be eligible for caching. */
    public static final long DEFAULT_MAX_CACHEABLE_SIZE = 100L * 1024 * 1024;

    /**
     * A cached resource: its bytes and the ETag computed once at load time.
     * The array is shared with every caller and must not be modified.
     *
     * @param data the resource bytes
     * @param etag the quoted ETag value for these bytes
     */
    public record Entry(byte[] data, String etag) {}

    /** Marker stored for paths that are known not to exist. */
    private static final Entry MISSING = new Entry(null, null);

    private final ResourcesLoader loader;
    private final long maxCacheableSize;
    private final ConcurrentMap<String, Entry> cache = new ConcurrentHashMap<>();

    /**
     * Creates a new ResourceCache with the given ResourceLoader and the default
     * size limit of {@link #DEFAULT_MAX_CACHEABLE_SIZE}.
     *
     * @param loader the resource loader to use
     */
    public ResourcesCache(ResourcesLoader loader) {
        this(loader, DEFAULT_MAX_CACHEABLE_SIZE);
    }

    /**
     * Creates a new ResourceCache with the given ResourceLoader and size limit.
     *
     * @param loader             the resource loader to use
     * @param maxCacheableSize   maximum resource size in bytes that may be
     *                           cached; larger resources are served but not
     *                           retained
     * @throws IllegalArgumentException if {@code maxCacheableSize} is negative
     */
    public ResourcesCache(ResourcesLoader loader, long maxCacheableSize) {
        if (maxCacheableSize < 0) {
            throw new IllegalArgumentException("maxCacheableSize must not be negative");
        }
        this.loader = loader;
        this.maxCacheableSize = maxCacheableSize;
    }

    /**
     * Gets a resource entry, caching it subject to the default cache policy.
     *
     * @param path the path to the resource
     * @return the cached entry, or null if the resource does not exist
     */
    public Entry get(String path) {
        return get(path, CachePolicy.CACHE);
    }

    /**
     * Gets a resource entry, honouring the given cache policy and the size limit.
     *
     * @param path   the path to the resource
     * @param policy whether the resource may be cached
     * @return the entry, or null if the resource does not exist
     */
    public Entry get(String path, CachePolicy policy) {
        boolean cacheable = policy == CachePolicy.CACHE;

        if (cacheable) {
            Entry cached = cache.get(path);
            if (cached != null) {
                return cached == MISSING ? null : cached;
            }
        }

        // Loading happens outside computeIfAbsent so blocking I/O never runs
        // while a ConcurrentHashMap bin is locked.
        byte[] data = loader.loadBinary(path);

        if (data == null) {
            if (cacheable) {
                cache.put(path, MISSING);
            }
            return null;
        }

        Entry entry = new Entry(data, Etags.of(data));
        if (cacheable && data.length <= maxCacheableSize) {
            cache.put(path, entry);
        }
        return entry;
    }

    /**
     * Gets binary data, honouring the given cache policy.
     *
     * @param path   the path to the resource
     * @param policy whether the resource may be cached
     * @return byte array or null if not found
     */
    public byte[] getBinary(String path, CachePolicy policy) {
        Entry entry = get(path, policy);
        return entry != null ? entry.data() : null;
    }

    /**
     * Gets binary data from cache, loading it lazily if not present.
     *
     * @param path the path to the resource
     * @return byte array or null if not found
     */
    public byte[] getBinary(String path) {
        return getBinary(path, CachePolicy.CACHE);
    }

    /**
     * Gets the resource decoded as UTF-8 text, honouring the given cache policy.
     *
     * @param path   the path to the resource
     * @param policy whether the resource may be cached
     * @return string or null if not found
     */
    public String getText(String path, CachePolicy policy) {
        byte[] data = getBinary(path, policy);
        return data != null ? new String(data, StandardCharsets.UTF_8) : null;
    }

    /**
     * Gets the resource decoded as UTF-8 text.
     *
     * @param path the path to the resource
     * @return string or null if not found
     */
    public String getText(String path) {
        return getText(path, CachePolicy.CACHE);
    }

    /**
     * The maximum resource size in bytes that may be cached.
     *
     * @return the size limit
     */
    public long maxCacheableSize() {
        return maxCacheableSize;
    }

    /**
     * Clears all cached resources.
     */
    public void clear() {
        cache.clear();
    }

    /**
     * Checks if a resource lookup is cached (including a cached miss).
     *
     * @param path the path to the resource
     * @return true if cached, false otherwise
     */
    public boolean isCached(String path) {
        return cache.containsKey(path);
    }

    /**
     * Check exist resource on disk or in jar file.
     *
     * @param path path to file
     * @return true exist, false don't exist
     */
    public boolean exists(String path) {
        return get(path) != null;
    }

    /**
     * Returns the resources available under the given directory.
     *
     * @param basePath path to folder
     * @return list of resource paths
     */
    public List<String> listResources(String basePath) {
        return loader.listResources(basePath);
    }

}
