package com.jazz.conlang;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Bean;

import com.jazz.conlang.app.WicketApplication;
import com.jazz.conlang.repo.TranslationRepository;

@SpringBootApplication
public class conlangApplication {
    public static void main(String[] args) {
        SpringApplication.run(conlangApplication.class, args);
    }

    @Bean
    public WicketApplication wicketApplication(TranslationRepository repo) {
        return new WicketApplication(repo);
    }
}
