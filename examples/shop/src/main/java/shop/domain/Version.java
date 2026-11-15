package shop.domain;

import lombok.Value;

@Value
public class Version {
    int major;
    int minor;

    public boolean isCompatibleWith(Version required) {
        return major == required.major && minor >= required.minor;
    }
}
