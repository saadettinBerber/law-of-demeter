package shop.service;

import shop.domain.Config;
import shop.domain.Settings;

public class Connector {
    public int portOf(Config config) {
        return config.server().port();
    }

    public int gatewayPortOf(Settings settings) {
        return settings.gateway().port();
    }
}
