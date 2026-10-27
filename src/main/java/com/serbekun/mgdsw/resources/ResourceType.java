package com.serbekun.mgdsw.resources;

public enum ResourceType {
    /** Static files like html css js images. */
    STATIC(),
    /** Content files that can be changes without update server code. */
    CONTENT(),  
    /** share files like minecraft maps zip archives that also like a {@link ResourceType#CONTENT} can be changes without update server code. */
    SHARED();
}
