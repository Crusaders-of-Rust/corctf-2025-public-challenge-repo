from flask import Flask, request, redirect, render_template, session, g

import sqlite3
import os
import re
import secrets

try:
    from secret import FLAG, ADMIN_USERNAME, ADMIN_PASSWORD
except (ImportError, AttributeError):
    FLAG = 'I want to report this fake flag: corCTF{EXAMPLE_FLAG}'
    ADMIN_USERNAME = 'admin'
    ADMIN_PASSWORD = 'password'

app = Flask(__name__)
app.secret_key = os.urandom(32)

DATABASE = 'msbug.db'

def get_db():
    if 'db' not in g:
        g.db = sqlite3.connect(DATABASE)
        g.db.row_factory = sqlite3.Row
    return g.db

@app.teardown_appcontext
def close_db(exception=None):
    db = g.pop('db', None)
    if db is not None:
        db.close()

def init_db():
    with app.app_context():
        db = get_db()
        db.execute('''
            CREATE TABLE IF NOT EXISTS reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                handled BOOLEAN NOT NULL DEFAULT 0
            )
        ''')

        existing = db.execute('SELECT COUNT(*) FROM reports').fetchone()[0]
        if existing == 0:
            db.execute(
                'INSERT INTO reports (content, handled) VALUES (?, ?)',
                (FLAG, 1)
            )
        db.commit()


@app.before_request
def add_nonce():
    if not getattr(request, 'csp_nonce', None):
        request.csp_nonce = secrets.token_hex(16)[:16]

@app.after_request
def set_csp(response):
    csp = (
        "default-src 'none'; "
        f"script-src 'nonce-{request.csp_nonce}'; "
        "style-src 'self'; "
        "img-src 'self'; "
        "connect-src 'self'; "
        "font-src 'none'; "
        "frame-ancestors 'none'; "
        "base-uri 'none'; "
        "object-src 'none'; "
    )
    response.headers['Content-Security-Policy'] = csp
    response.headers['X-Content-Type-Options'] = 'nosniff'
    response.headers['X-Frame-Options'] = 'SAMEORIGIN'
    response.headers['X-XSS-Protection'] = '1; mode=block'
    response.headers['Referrer-Policy'] = 'strict-origin-when-cross-origin'
    return response

@app.route('/', methods=['GET', 'POST'])
def index():
    error = None
    if request.method == 'POST':
        content = request.form.get('content', '').strip()
        if len(content) == 0 or len(content) > 120:
            error = 'Bug report must be between 1 and 120 characters.'
        if web_application_firewall(content):
            error = 'Please dont try to hack us :C'
        else:
            db = get_db()
            db.execute('INSERT INTO reports (content, handled) VALUES (?, ?)', (content, 0))
            db.commit()
            return redirect('/')
    return render_template('index.html', error=error)


@app.route('/login', methods=['GET', 'POST'])
def login():
    error = None
    if request.method == 'POST':
        username = request.form.get('username', '')
        password = request.form.get('password', '')
        if username == ADMIN_USERNAME and password == ADMIN_PASSWORD:
            session['logged_in'] = True
            return redirect('/admin')
        else:
            error = 'Invalid credentials.'
    return render_template('login.html', error=error)


@app.route('/logout')
def logout():
    session.pop('logged_in', None)
    return redirect('/')


@app.route('/admin')
def admin():
    if not session.get('logged_in'):
        return redirect('/login')
    db = get_db()
    reports = db.execute('SELECT id, content, handled FROM reports ORDER BY id DESC').fetchall()
    return render_template('admin.html', reports=reports, csp_nonce=request.csp_nonce)


@app.route('/handle/<int:report_id>', methods=['POST'])
def handle_report(report_id):
    if not session.get('logged_in'):
        return redirect('/login')

    db = get_db()
    db.execute('UPDATE reports SET handled = 1 WHERE id = ?', (report_id,))
    db.commit()
    return redirect('/admin')


def web_application_firewall(content: str) -> bool:

    dangerous_tags = [
        'script', 'iframe', 'object', 'embed', 'link', 'style', 'meta', 'base', 'img', 'video', 'audio'
    ]
    tag_pattern = re.compile(r'</?\s*({})\b'.format('|'.join(dangerous_tags)))
    if tag_pattern.search(content.lower()):
        return True

    on_event_pattern = re.compile(r'\bon\w+\s*=', re.IGNORECASE)
    if on_event_pattern.search(content.lower()):
        return True

    return False


if __name__ == '__main__':
    if not os.path.exists(DATABASE):
        init_db()
    app.run()
else:
    with app.app_context():
        if not os.path.exists(DATABASE):
            init_db()
