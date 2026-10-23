package com.serbekun.mgdsw.resources;

/**
 * Whether the resources of a {@link ResourcesBasePath} directory are shipped inside the
 * JAR and unpacked on first run, or exist only on disk.
 *
 * <p>Each {@link ResourcesBasePath} constant carries one of these values, so the decision
 * is made once per resource directory rather than at every lookup. It is consumed in two
 * places:
 *
 * <ul>
 *   <li>{@link ResourcesBasePath#isPackaged()} maps the policy to a boolean, and
 *       {@link ResourcesBasePath#packagedDirectories()} uses it to pick which directories
 *       {@link ResourcesUnpacker#unpack()} copies out of the classpath;</li>
 *   <li>directories that are not packaged are still created on disk by
 *       {@link ResourcesUnpacker} via {@link ResourcesBasePath#directoriesToCreate()}, so
 *       the operator has somewhere to drop files without the JAR ever providing them.</li>
 * </ul>
 *
 * <p>Note that this is not {@link LookupOrder}: the policy describes what ships in the
 * artifact, while the lookup order decides which copy wins at request time.
 */
public enum UnpackPolicy {

    /**
     * The directory's contents are templates bundled in the JAR and copied to disk on first
     * run (and after a version upgrade). Files already on disk are never overwritten.
     */
    PACKAGED,

    /**
     * Nothing is shipped for this directory; it is created empty on disk and populated by
     * the operator. {@link ResourcesUnpacker} never copies files into it.
     */
    DISK_ONLY
}