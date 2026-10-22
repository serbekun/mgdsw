package com.serbekun.mgdsw;

import com.serbekun.mgdsw.services.resources.ResourcesService;
import com.serbekun.mgdsw.http.InitHttp;
import com.serbekun.mgdsw.resources.ResourcesCache;
import com.serbekun.mgdsw.resources.ResourcesLoader;
import com.serbekun.mgdsw.resources.ResourcesUnpacker;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class Main {

    private static final Logger log = LoggerFactory.getLogger(Main.class);

    public static void main(String[] args) {

        log.info("mgdsw" + BuildInfo.version());

        // Create ResourceService
        ResourcesLoader resourcesLoader = new ResourcesLoader();
        ResourcesCache resourcesCache = new ResourcesCache(resourcesLoader);
        ResourcesService resourcesService = new ResourcesService(resourcesCache);

        // Create unpacker and unpack resources.
        ResourcesUnpacker resourcesUnpacker = new ResourcesUnpacker(resourcesLoader, BuildInfo.version());
        resourcesUnpacker.unpack();

        InitHttp initHttp = new InitHttp();
        initHttp.init();
    }
}
