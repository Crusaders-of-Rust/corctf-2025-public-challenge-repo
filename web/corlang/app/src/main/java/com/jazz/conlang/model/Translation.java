package com.jazz.conlang.model;

import java.io.Serializable;

import org.apache.wicket.Component;
import org.apache.wicket.model.IModel;
import org.apache.wicket.model.Model;
import org.apache.wicket.model.StringResourceModel;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.Id;

@Entity
public class Translation implements Serializable {
    @Id
    @GeneratedValue
    private Long id;

    private String keyName;
    private String localeTag;
    @Column(name = "translation_value")
    private String value;
    private boolean approved;
    private String providedBy;

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public String getKeyName() {
        return keyName;
    }

    public void setKeyName(String keyName) {
        this.keyName = keyName;
    }

    public String getLocaleTag() {
        return localeTag;
    }

    public void setLocaleTag(String localeTag) {
        this.localeTag = localeTag;
    }

    public String getValue() {
        return value;
    }

    public void setValue(String value) {
        this.value = value;
    }

    public boolean isApproved() {
        return approved;
    }

    public void setApproved(boolean approved) {
        this.approved = approved;
    }

    public String getAuthor() {
        return providedBy;
    }

    public void setProvidedBy(String providedBy) {
        this.providedBy = providedBy;
    }

    /**
     * Creates a Wicket model that renders this translation.
     * 
     * @param context The Wicket component or page to use for property resolution.
     */
    public IModel<String> render(Component context) {
        return new StringResourceModel(this.keyName, context, Model.of(context))
                .setDefaultValue(this.value);
    }
}
