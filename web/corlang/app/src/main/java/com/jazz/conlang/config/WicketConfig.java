package com.jazz.conlang.config;

import org.apache.wicket.protocol.http.WicketFilter;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.web.servlet.FilterRegistrationBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import com.jazz.conlang.app.WicketApplication;
import com.jazz.conlang.repo.TranslationRepository;

@Configuration
public class WicketConfig {

    @Autowired
    private TranslationRepository translationRepository;

    @Bean
    public FilterRegistrationBean<WicketFilter> wicketFilter() {
        FilterRegistrationBean<WicketFilter> registration = new FilterRegistrationBean<>();
        registration.setFilter(new WicketFilter(new WicketApplication(translationRepository)));
        registration.addUrlPatterns("/*");
        registration.setName("WicketFilter");
        registration.addInitParameter(WicketFilter.APP_FACT_PARAM,
                "com.jazz.conlang.app.WicketApplication");
        registration.addInitParameter(WicketFilter.FILTER_MAPPING_PARAM, "/*");
        registration.setOrder(1);
        return registration;
    }
}