package com.serbekun.mgdsw;

import com.serbekun.mgdsw.services.resources.ResourcesService;
import com.serbekun.mgdsw.config.ConfigContext;
import com.serbekun.mgdsw.config.ConfigLoader;
import com.serbekun.mgdsw.http.InitHttp;
import com.serbekun.mgdsw.resources.ResourceType;
import com.serbekun.mgdsw.resources.ResourcesCache;
import com.serbekun.mgdsw.resources.ResourcesLoader;
import com.serbekun.mgdsw.resources.ResourcesUnpacker;

import java.nio.file.Path;
import java.util.Map;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class Main {

    private static final Logger log = LoggerFactory.getLogger(Main.class);

    public static void main(String[] args) {

        log.info("mgdsw" + BuildInfo.version());

        // Create config objects
        log.info("starting init config.");
        ConfigContext config = null;

        ConfigLoader configLoader = new ConfigLoader();

        
        String configFilePath = System.getenv(Envs.MGDSW_CONFIG.name());
        if (configFilePath == null) {
            config = configLoader.loadDefault();
        } else {
            Path configFilePathOf = Path.of(configFilePath);
            config = configLoader.load(configFilePathOf);
        }

        // Create ResourceService
        log.info("starting resource mod.");
        Path staticRoot = Path.of(config.configResources().staticBasePath());
        ResourcesLoader resourcesLoader = new ResourcesLoader(staticRoot);
        ResourcesCache resourcesCache = new ResourcesCache(
                Map.of(
                        ResourceType.STATIC, staticRoot,
                        ResourceType.SHARED, Path.of(config.configResources().sharedBasePath())),
                config.configResources().maxCacheableSize());
        ResourcesService resourcesService = new ResourcesService(resourcesCache);

        // Create unpacker and unpack resources.
        ResourcesUnpacker resourcesUnpacker = new ResourcesUnpacker(resourcesLoader, BuildInfo.version());
        resourcesUnpacker.unpack(config.configResources());

        // init http layer
        log.info("starting init http layer.");
        InitHttp http = new InitHttp(config.http());
        http.init();
    }
}
