package com.jazz.conlang.service;

import java.util.Locale;
import java.util.Optional;

import org.apache.wicket.Component;
import org.apache.wicket.resource.loader.IStringResourceLoader;

import com.jazz.conlang.repo.TranslationRepository;

public class DatabaseStringResourceLoader implements IStringResourceLoader {

    private final TranslationRepository repo;

    public DatabaseStringResourceLoader(TranslationRepository repo) {
        this.repo = repo;
    }

    @Override
    public String loadStringResource(Class<?> clazz, String key, Locale locale, String style, String variation) {
        if (key == null || locale == null)
            return null;
        Optional<com.jazz.conlang.model.Translation> t = repo.findByKeyNameAndLocaleTagAndApprovedTrue(key,
                locale.getLanguage());
        return t.map(com.jazz.conlang.model.Translation::getValue).orElse(null);
    }

    @Override
    public String loadStringResource(Component component, String key, Locale locale, String style, String variation) {
        return loadStringResource((Class<?>) null, key, locale, style, variation);
    }
}
