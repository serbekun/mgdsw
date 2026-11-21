package com.serbekun.mgdsw;

import static org.assertj.core.api.Assertions.assertThatCode;

import org.junit.jupiter.api.Test;

class MainTest {
    @Test
    void applicationStarts() {
        assertThatCode(() -> Main.main(new String[0]))
                .doesNotThrowAnyException();
    }
}
