package com.jazz.conlang.pages;

import org.apache.wicket.authroles.authorization.strategies.role.annotations.AuthorizeInstantiation;
import org.apache.wicket.markup.html.WebPage;
import org.apache.wicket.markup.html.basic.Label;
import org.apache.wicket.markup.html.form.Form;
import org.apache.wicket.markup.html.form.TextField;
import org.apache.wicket.markup.html.link.Link;
import org.apache.wicket.markup.html.panel.FeedbackPanel;
import org.apache.wicket.model.PropertyModel;
import org.apache.wicket.spring.injection.annot.SpringBean;

import com.jazz.conlang.app.AuthenticatedSession;
import com.jazz.conlang.model.Translation;
import com.jazz.conlang.repo.TranslationRepository;
import com.jazz.conlang.util.CorLangGenerator;

@AuthorizeInstantiation("USER")
public class HomePage extends WebPage {

    @SpringBean
    private TranslationRepository repo;

    private String translation;
    private String corWord;

    public HomePage() {
        corWord = CorLangGenerator.generate();

        add(new Label("corWord", corWord));

        add(new Label("karma", 
            ((AuthenticatedSession) AuthenticatedSession.get()).getKarma()
        ));

        Form<Void> form = new Form<Void>("form") {
            @Override
            protected void onSubmit() {
                Translation t = new Translation();
                t.setKeyName(corWord);
                t.setLocaleTag(AuthenticatedSession.get().getLocale().getLanguage());
                t.setValue(translation);
                t.setApproved(false);
                t.setProvidedBy(((AuthenticatedSession) AuthenticatedSession.get()).getUsername());
                repo.save(t);
                Long id = t.getId();
                info("Submitted for approval! Your translation has ID: " + id);
            }
        };
        form.add(new TextField<>("translation", new PropertyModel<>(this, "translation")));
        add(form);

        add(new Link<Void>("logout") {
            @Override
            public void onClick() {
                AuthenticatedSession.get().invalidate();
                setResponsePage(LoginPage.class);
            }
        });

        add(new FeedbackPanel("feedback"));
    }
}
