package com.jazz.conlang.repo;

import java.util.List;
import java.util.Optional;

import org.springframework.data.jpa.repository.JpaRepository;

import com.jazz.conlang.model.Translation;

public interface TranslationRepository extends JpaRepository<Translation, Long> {
    Optional<Translation> findByKeyNameAndLocaleTagAndApprovedTrue(String keyName, String localeTag);

    List<Translation> findByApprovedFalse();
}
