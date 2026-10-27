package com.serbekun.mgdsw.resources;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;

/**
 * Thread-safe cache layer for resources, keyed by {@link ResourceType} and path.
 *
 * <p>The resource type is part of the contract, not just metadata:
 *
 * <ul>
 *   <li>{@link ResourceType#STATIC} is read from the static root with the usual
 *       classpath fallback and may be cached;</li>
 *   <li>{@link ResourceType#CONTENT} is never served by this class. Content is
 *       configuration data owned by its own services, not an HTTP resource.</li>
 *   <li>{@link ResourceType#SHARED} is read from the shared root and is
 *       <strong>never cached</strong>, even when the caller asks for
 *       {@link CachePolicy#CACHE}. Shared files are large and are served by
 *       nginx, so retaining their bytes in the JVM heap would be a bug.</li>
 * </ul>
 *
 * <p>Only bytes are cached — the server answers with bytes, so keeping a decoded
 * copy would both duplicate memory and re-encode on every response. Text is
 * decoded on demand, always as UTF-8.
 *
 * <p>Misses are cached too (for cacheable types): a bot scanning for files that
 * do not exist must not reach disk and classloader on every request.
 *
 * <p>Independently of the policy, resources larger than
 * {@link #maxCacheableSize()} are returned but not cached, so a large file
 * cannot retain its bytes in memory forever.
 */
public class ResourcesCache {

    /** Default upper bound (100 MiB) for a resource to be eligible for caching. */
    public static final long DEFAULT_MAX_CACHEABLE_SIZE = 100L * 1024 * 1024;

    /**
     * A loaded resource: its bytes and, for cacheable types, the ETag computed
     * once at load time. The array is shared with every caller and must not be
     * modified.
     *
     * @param data the resource bytes
     * @param etag the quoted ETag value, or {@code null} for {@link ResourceType#SHARED}
     *             entries, which are never cached and never hashed
     */
    public record Entry(byte[] data, String etag) {}

    /** Marker stored for paths that are known not to exist. */
    private static final Entry MISSING = new Entry(null, null);

    private final Map<ResourceType, ResourcesLoader> loaders;
    private final long maxCacheableSize;
    private final ConcurrentMap<Key, Entry> cache = new ConcurrentHashMap<>();

    /** Cache key that keeps equal relative paths in different roots apart. */
    private record Key(ResourceType type, String path) {}

    /**
     * Creates a cache with an explicit root per served resource type.
     *
     * <p>A {@link ResourceType#STATIC} root is required. A
     * {@link ResourceType#SHARED} root is optional: when it is absent, shared
     * lookups fail with a clear error instead of silently reading from static.
     * A {@link ResourceType#CONTENT} root is deliberately not used.
     *
     * @param roots            on-disk roots for the types this cache serves
     * @param maxCacheableSize maximum resource size in bytes that may be cached
     * @throws IllegalArgumentException if a required root is missing, the static
     *                                  and shared roots are not separate, or the
     *                                  size limit is negative
     */
    public ResourcesCache(Map<ResourceType, Path> roots, long maxCacheableSize) {
        if (maxCacheableSize < 0) {
            throw new IllegalArgumentException("maxCacheableSize must not be negative");
        }
        if (roots == null) {
            throw new IllegalArgumentException("roots must not be null");
        }

        Path staticRoot = roots.get(ResourceType.STATIC);
        if (staticRoot == null) {
            throw new IllegalArgumentException("A STATIC root is required");
        }

        Path sharedRoot = roots.get(ResourceType.SHARED);
        if (sharedRoot != null && (sharedRoot.equals(staticRoot)
                || sharedRoot.startsWith(staticRoot)
                || staticRoot.startsWith(sharedRoot))) {
            throw new IllegalArgumentException(
                    "STATIC and SHARED roots must be separate, non-nested directories: "
                            + staticRoot + " vs " + sharedRoot);
        }

        Map<ResourceType, ResourcesLoader> configured = new HashMap<>();
        configured.put(ResourceType.STATIC,
                new ResourcesLoader(staticRoot, LookupOrder.DISK_FIRST));
        if (sharedRoot != null) {
            configured.put(ResourceType.SHARED,
                    new ResourcesLoader(sharedRoot, LookupOrder.DISK_ONLY));
        }
        this.loaders = Map.copyOf(configured);
        this.maxCacheableSize = maxCacheableSize;
    }

    // region typed lookup

    /**
     * Gets a resource entry, honouring the type's cache rules and the given
     * policy.
     *
     * @param type   the resource type
     * @param path   the path to the resource, relative to the type's root
     * @param policy whether a cacheable resource may be cached; ignored for
     *               {@link ResourceType#SHARED}
     * @return the entry, or null if the resource does not exist or the path is empty
     * @throws IllegalArgumentException for {@link ResourceType#CONTENT}
     */
    public Entry get(ResourceType type, String path, CachePolicy policy) {
        if (type == ResourceType.CONTENT) {
            throw new IllegalArgumentException(
                    "CONTENT resources are not served by ResourcesCache");
        }
        if (path == null || path.isEmpty()) {
            return null;
        }

        boolean cacheable = type != ResourceType.SHARED && policy == CachePolicy.CACHE;
        Key key = new Key(type, path);

        if (cacheable) {
            Entry cached = cache.get(key);
            if (cached != null) {
                return cached == MISSING ? null : cached;
            }
        }

        // Loading happens outside computeIfAbsent so blocking I/O never runs
        // while a ConcurrentHashMap bin is locked.
        byte[] data = loader(type).loadBinary(path);

        if (data == null) {
            if (cacheable) {
                cache.put(key, MISSING);
            }
            return null;
        }

        // SHARED bytes are never retained and never hashed: hashing a 500 MB
        // archive on every lookup would defeat the point of not caching it.
        Entry entry = new Entry(data,
                type == ResourceType.SHARED ? null : Etags.of(data));
        if (cacheable && data.length <= maxCacheableSize) {
            cache.put(key, entry);
        }
        return entry;
    }

    /**
     * Gets binary data, honouring the type's cache rules and the given policy.
     *
     * @param type   the resource type
     * @param path   the path to the resource
     * @param policy whether a cacheable resource may be cached
     * @return byte array or null if not found
     */
    public byte[] getBinary(ResourceType type, String path, CachePolicy policy) {
        Entry entry = get(type, path, policy);
        return entry != null ? entry.data() : null;
    }

    /**
     * Gets the resource decoded as UTF-8 text, honouring the type's cache rules.
     *
     * @param type   the resource type
     * @param path   the path to the resource
     * @param policy whether a cacheable resource may be cached
     * @return string or null if not found
     */
    public String getText(ResourceType type, String path, CachePolicy policy) {
        byte[] data = getBinary(type, path, policy);
        return data != null ? new String(data, StandardCharsets.UTF_8) : null;
    }

    /**
     * Checks whether a resource exists without retaining its bytes for
     * {@link ResourceType#SHARED}.
     *
     * @param type the resource type
     * @param path the path to the resource
     * @return true when the resource exists
     */
    public boolean exists(ResourceType type, String path) {
        if (type == ResourceType.CONTENT) {
            throw new IllegalArgumentException(
                    "CONTENT resources are not served by ResourcesCache");
        }
        if (path == null || path.isEmpty()) {
            return false;
        }
        if (type == ResourceType.SHARED) {
            return loader(type).exists(path);
        }
        return get(type, path, CachePolicy.CACHE) != null;
    }

    /**
     * Returns the resources available under the given directory of a type.
     *
     * @param type     the resource type
     * @param basePath path to the folder, relative to the type's root
     * @return list of resource paths
     * @throws IllegalArgumentException for {@link ResourceType#CONTENT}
     */
    public List<String> listResources(ResourceType type, String basePath) {
        if (type == ResourceType.CONTENT) {
            throw new IllegalArgumentException(
                    "CONTENT resources are not served by ResourcesCache");
        }
        return loader(type).listResources(basePath);
    }

    /**
     * Checks if a resource lookup is cached, including a cached miss.
     *
     * @param type the resource type
     * @param path the path to the resource
     * @return true if cached, false otherwise
     */
    public boolean isCached(ResourceType type, String path) {
        if (type == ResourceType.CONTENT || path == null || path.isEmpty()) {
            return false;
        }
        return cache.containsKey(new Key(type, path));
    }

    // endregion

    // region static shortcuts

    /** Gets a {@link ResourceType#STATIC} entry, cached subject to the default policy. */
    public Entry get(String path) {
        return get(ResourceType.STATIC, path, CachePolicy.CACHE);
    }

    /** Gets a {@link ResourceType#STATIC} entry, honouring the given policy. */
    public Entry get(String path, CachePolicy policy) {
        return get(ResourceType.STATIC, path, policy);
    }

    /** Gets {@link ResourceType#STATIC} binary data with the default policy. */
    public byte[] getBinary(String path) {
        return getBinary(ResourceType.STATIC, path, CachePolicy.CACHE);
    }

    /** Gets {@link ResourceType#STATIC} binary data, honouring the given policy. */
    public byte[] getBinary(String path, CachePolicy policy) {
        return getBinary(ResourceType.STATIC, path, policy);
    }

    /** Gets a {@link ResourceType#STATIC} resource as UTF-8 text. */
    public String getText(String path) {
        return getText(ResourceType.STATIC, path, CachePolicy.CACHE);
    }

    /** Gets a {@link ResourceType#STATIC} resource as UTF-8 text, honouring the given policy. */
    public String getText(String path, CachePolicy policy) {
        return getText(ResourceType.STATIC, path, policy);
    }

    /** Checks whether a {@link ResourceType#STATIC} resource exists. */
    public boolean exists(String path) {
        return exists(ResourceType.STATIC, path);
    }

    /** Checks whether a {@link ResourceType#STATIC} lookup is cached. */
    public boolean isCached(String path) {
        return isCached(ResourceType.STATIC, path);
    }

    /** Lists {@link ResourceType#STATIC} resources under the given directory. */
    public List<String> listResources(String basePath) {
        return listResources(ResourceType.STATIC, basePath);
    }

    // endregion

    /**
     * The maximum resource size in bytes that may be cached.
     *
     * @return the size limit
     */
    public long maxCacheableSize() {
        return maxCacheableSize;
    }

    /** Clears all cached resources. */
    public void clear() {
        cache.clear();
    }

    private ResourcesLoader loader(ResourceType type) {
        ResourcesLoader loader = loaders.get(type);
        if (loader == null) {
            throw new IllegalStateException("No root configured for resource type " + type);
        }
        return loader;
    }
}
