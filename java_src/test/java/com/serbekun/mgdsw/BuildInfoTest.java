package com.serbekun.mgdsw;

import static org.assertj.core.api.Assertions.assertThat;

import org.junit.jupiter.api.Test;

class BuildInfoTest {
    @Test
    void versionIsAvailable() {
        assertThat(BuildInfo.version()).isNotBlank().isNotEqualTo("unknown");
    }
}
