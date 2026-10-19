package com.serbekun.mgdsw;

import com.serbekun.mgdsw.services.resources.ResourcesService;
import com.serbekun.mgdsw.resources.ResourcesCache;
import com.serbekun.mgdsw.resources.ResourcesLoader;

public final class Main {

    public static void main(String[] args) {
        System.out.println("mgdsw " + BuildInfo.version());

        // Create ResourceService
        ResourcesLoader resourcesLoader = new ResourcesLoader();
        ResourcesCache resourcesCache = new ResourcesCache(resourcesLoader);
        ResourcesService resourcesService = new ResourcesService(resourcesCache);

    }
}
