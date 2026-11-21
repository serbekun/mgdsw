package com.serbekun.mgdsw.services.sharedcontent;

import com.serbekun.mgdsw.domain.http.ApiV0SharedMinecraftMapsYamlRes;
import com.serbekun.mgdsw.domain.sharedcontent.MinecraftMap;
import com.serbekun.mgdsw.domain.sharedcontent.MinecraftMaps;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/**
 * Public API and business rules for Minecraft map metadata.
 * <p>
 * Disk access, slug listing and YAML parsing are delegated to
 * {@link MinecraftMaps}. This service decides which maps are visible to the
 * outside world and converts stored objects into public response objects.
 * </p>
 */
public class MinecraftMapsService {

    private static final Logger log = LoggerFactory.getLogger(MinecraftMapsService.class);

    private final MinecraftMaps minecraftMaps;

    /**
     * Cached snapshot of the slugs of published maps.
     * <p>
     * The list is always immutable and is never modified in place: a rebuild
     * creates a new list and replaces the reference. Together with
     * {@code volatile} this means readers see either the old or the new
     * snapshot, never a partially built one.
     * </p>
     */
    private volatile List<String> minecraftMapsPublicSlugList = List.of();

    /**
     * Creates a service backed by an existing map store and immediately builds
     * the cached list of published slugs, so that
     * {@link #getMinecraftMapsPublicList()} is usable right after construction.
     * <p>
     * Building the list reads and parses the YAML of every map, so construction
     * cost grows with the number of maps.
     * </p>
     *
     * @param minecraftMaps the Minecraft map metadata store, must not be {@code null}
     * @throws NullPointerException if {@code minecraftMaps} is {@code null}
     */
    public MinecraftMapsService(MinecraftMaps minecraftMaps) {
        this.minecraftMaps = Objects.requireNonNull(minecraftMaps, "minecraftMaps");
        buildMinecraftMapsPublicSlugList();
    }

    /**
     * Rebuilds the cached list of published slugs from the store.
     * <p>
     * Walks every slug returned by {@link MinecraftMaps#getSlugs()}, keeps only
     * the maps that exist and are published, and atomically replaces the
     * cached snapshot with the result.
     * </p>
     */
    private void buildMinecraftMapsPublicSlugList() {
        List<String> publicSlugs = new ArrayList<>();

        for (String slug : minecraftMaps.getSlugs()) {
            MinecraftMap minecraftMap = minecraftMaps.getBySlug(slug);
            if (minecraftMap != null && minecraftMap.published()) {
                publicSlugs.add(slug);
            }
        }

        minecraftMapsPublicSlugList = List.copyOf(publicSlugs);
    }

    // region public api

    /**
     * Re-scans the content root and refreshes the cached list of published
     * slugs.
     * <p>
     * First asks the underlying {@link MinecraftMaps} to rebuild its own slug
     * list, then rebuilds the public snapshot from it. The method is
     * synchronized so that two concurrent rebuilds cannot publish their
     * results out of order; readers of {@link #getMinecraftMapsPublicList()}
     * are not blocked and keep seeing the previous snapshot until the new one
     * is ready.
     * </p>
     */
    public synchronized void rebuildMinecraftMapsSlugList() {
        minecraftMaps.rebuildMinecraftMapsSlugList();
        buildMinecraftMapsPublicSlugList();
    }

    /**
     * Returns a {@link ApiV0SharedMinecraftMapsYamlRes} object if the map exists
     * and is published.
     *
     * @param slug minecraft map slug.
     * @return {@link ApiV0SharedMinecraftMapsYamlRes} object, or {@code null} if
     *         the map does not exist or is not published.
     */
    public ApiV0SharedMinecraftMapsYamlRes getMinecraftMapPublic(String slug) {
        MinecraftMap minecraftMap = minecraftMaps.getBySlug(slug);

        // if minecraft map not published. from outside you can't get data.
        if (minecraftMap == null || !minecraftMap.published()) {
            log.info("Minecraft map: {} not found or not published", slug);
            return null;
        }

        return convertInternalMinecraftObjToSharedObj(minecraftMap);
    }

    /**
     * All map slugs found in the content root, <b>including unpublished
     * maps</b>.
     * <p>
     * This is the raw list as returned by the store and it is not filtered by
     * visibility, so it must not be exposed to outside callers. Use
     * {@link #getMinecraftMapsPublicList()} for the list that is safe to
     * publish.
     * </p>
     *
     * @return the slugs of all maps, published or not
     */
    public List<String> getMinecraftMapsList() {
        return minecraftMaps.getSlugs();
    }

    /**
     * Slug list exposed to the public API, i.e. the maps that are published and
     * therefore visible from outside.
     * <p>
     * Unlike {@link #getMinecraftMapsList()}, which returns every slug found in
     * the content root, this list is a cached snapshot built at construction
     * time and refreshed by {@link #rebuildMinecraftMapsSlugList()}. The
     * returned list is immutable.
     * </p>
     *
     * @return the published map slugs, never {@code null}
     */
    public List<String> getMinecraftMapsPublicList() {
        // Safe to return directly: the snapshot is immutable (List.copyOf)
        // and is replaced as a whole on rebuild, so no defensive copy is needed.
        return minecraftMapsPublicSlugList;
    }

    // endregion

    /**
     * Converts the internal map object into the public response object.
     * <p>
     * Fields are copied one to one; no visibility check is done here, the
     * caller must make sure the map is published.
     * </p>
     *
     * @param minecraftMap the internal map, must not be {@code null}
     * @return the public response object
     */
    // TODO(#25): object fields of MinecraftMap (authors, description, license,
    //  preview, etc.) may be null. Ensure the public response never leaks
    //  unexpected nulls and cover this mapping with tests (all fields mapped,
    //  order not mixed up).
    private static ApiV0SharedMinecraftMapsYamlRes convertInternalMinecraftObjToSharedObj(MinecraftMap minecraftMap) {
        return new ApiV0SharedMinecraftMapsYamlRes(
                minecraftMap.slug(),
                minecraftMap.name(),
                minecraftMap.authors(),
                minecraftMap.description(),
                minecraftMap.license(),
                minecraftMap.mapVersion(),
                minecraftMap.releaseDate(),
                minecraftMap.minecraftVersion(),
                minecraftMap.preview(),
                minecraftMap.zipSha256());
    }
}