package com.jazz.conlang.pages;

import java.util.List;

import org.apache.wicket.authroles.authorization.strategies.role.annotations.AuthorizeInstantiation;
import org.apache.wicket.markup.html.WebPage;
import org.apache.wicket.markup.html.basic.Label;
import org.apache.wicket.markup.html.link.BookmarkablePageLink;
import org.apache.wicket.markup.html.list.ListItem;
import org.apache.wicket.markup.html.list.ListView;
import org.apache.wicket.request.mapper.parameter.PageParameters;
import org.apache.wicket.spring.injection.annot.SpringBean;

import com.jazz.conlang.model.Translation;
import com.jazz.conlang.repo.TranslationRepository;

@AuthorizeInstantiation("ADMIN")
public class AdminPage extends WebPage {

    @SpringBean
    private TranslationRepository repo;

    public AdminPage() {
    }

    @Override
    protected void onInitialize() {
        super.onInitialize();

        List<Translation> pending = repo.findByApprovedFalse();

        add(new ListView<Translation>("pendingList", pending) {
            @Override
            protected void populateItem(ListItem<Translation> item) {
                Translation translation = item.getModelObject();
                item.add(new Label("keyName", translation.getKeyName()));
                item.add(new Label("locale", translation.getLocaleTag()));
                item.add(new Label("proposed", translation.getValue()));
                
                PageParameters params = new PageParameters();
                params.add("id", translation.getId());
                item.add(new BookmarkablePageLink<Void>("detail", TranslationDetailPage.class, params));
            }
        });
    }
}
