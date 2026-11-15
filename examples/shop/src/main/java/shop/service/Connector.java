package shop.service;

import shop.domain.Config;

public class Connector {
    public int portOf(Config config) {
        return config.server().port();
    }
}
