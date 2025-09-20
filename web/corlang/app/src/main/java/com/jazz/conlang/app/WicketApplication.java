package com.jazz.conlang.app;

import org.apache.wicket.Page;
import org.apache.wicket.authroles.authentication.AuthenticatedWebApplication;
import org.apache.wicket.authroles.authentication.AuthenticatedWebSession;
import org.apache.wicket.authroles.authorization.strategies.role.annotations.AnnotationsRoleAuthorizationStrategy;
import org.apache.wicket.csp.CSPDirective;
import org.apache.wicket.markup.html.WebPage;
import org.apache.wicket.spring.injection.annot.SpringComponentInjector;

import com.jazz.conlang.pages.AdminPage;
import com.jazz.conlang.pages.HomePage;
import com.jazz.conlang.pages.LoginPage;
import com.jazz.conlang.pages.TranslationDetailPage;
import com.jazz.conlang.repo.TranslationRepository;
import com.jazz.conlang.service.DatabaseStringResourceLoader;

public class WicketApplication extends AuthenticatedWebApplication {

    private final TranslationRepository translationRepo;

    public WicketApplication(TranslationRepository repo) {
        this.translationRepo = repo;
    }

    @Override
    public Class<? extends Page> getHomePage() {
        return HomePage.class;
    }

    @Override
    protected Class<? extends WebPage> getSignInPageClass() {
        return LoginPage.class;
    }

    @Override
    public Class<? extends AuthenticatedWebSession> getWebSessionClass() {
        return AuthenticatedSession.class;
    }

    @Override
    public void init() {
        super.init();

        // Enable Spring @SpringBean injection
        getComponentInstantiationListeners().add(new SpringComponentInjector(this));

        // Load translations from DB
        getResourceSettings().getStringResourceLoaders().add(0,
                new DatabaseStringResourceLoader(translationRepo));

        // Mount pages
        mountPage("/login", LoginPage.class);
        mountPage("/admin", AdminPage.class);
        mountPage("/admin/translation", TranslationDetailPage.class);

        // Enable annotation-based role checks (@AuthorizeInstantiation)
        getSecuritySettings().setAuthorizationStrategy(
                new AnnotationsRoleAuthorizationStrategy(this));

        // Allow Bootstrap in CSP
        getCspSettings().blocking().add(CSPDirective.STYLE_SRC,
                "https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css");
    }
}
