import io
import os
import piexif
import requests
import tempfile
import uuid

from PIL import Image, ExifTags
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import hashes, hmac
from flask import Flask, request, send_file, render_template
from urllib.parse import urljoin, urlparse
from werkzeug.utils import secure_filename
from secrets import token_hex

from celery_config import celery_app
from secret import API_TOKEN
from tasks import process_image

app = Flask(__name__)

celery_app.conf.update(app.config)

UPLOAD_FOLDER = 'uploads/'
ENCRYPTION_KEY = token_hex(32)


def hmac_sha256(data):
    h = hmac.HMAC(ENCRYPTION_KEY, hashes.SHA256(), backend=default_backend())
    h.update(data)
    return h.finalize().hex()


def encrypt_exif_data(exif_data):
    new_exif_data = {}
    for tag, value in exif_data.items():
        if tag in ExifTags.TAGS:
            tag_name = ExifTags.TAGS[tag]
            if tag_name == "Orientation":
                new_exif_data[tag] = 1
            else:
                new_exif_data[tag] = value
        else:
            new_exif_data[tag] = hmac_sha256(value)
    return new_exif_data


@app.route('/', methods=['GET', 'POST'])
def upload_file():
    if request.method == 'POST':
        file = request.files['file']
        if file:
            try:
                img = Image.open(file)
                if img.format != "JPEG":
                    return "Please upload a valid JPEG image.", 400

                exif_data = img._getexif()
                encrypted_exif = None
                if exif_data:
                    encrypted_exif = piexif.dump(encrypt_exif_data(exif_data))
                filename = secure_filename(file.filename)
                temp_path = os.path.join(tempfile.gettempdir(), filename)
                img.save(temp_path)

                unique_id = str(uuid.uuid4())
                new_file_path = os.path.join(UPLOAD_FOLDER, f"{unique_id}.png")
                process_image.apply_async(args=[temp_path, new_file_path, encrypted_exif])

                return render_template("processing.html", image_url=f"/anonymized/?uuid={unique_id}")

            except Exception as e:
                return f"Error: {e}", 400

    return render_template("index.html")


@app.route('/anonymized/')
def serve_image():
    image_uuid = ""
    try:
        image_uuid = request.args.get('uuid')
        url = create_file_url(image_uuid)

        resp = requests.get(url, headers={"Authorization": f"Token {API_TOKEN}"})
        if resp.status_code != 200:
            raise ValueError("File does not exist according to fileserver")

        return send_file(
            io.BytesIO(resp.content),
            mimetype="image/png",
            as_attachment=False,
            download_name=f"{image_uuid}.png"
        )
    except Exception as e:
        return f"Image {image_uuid} cannot be found: {e}.", 404


def create_file_url(uuid):
    file_url = urljoin("http://127.0.0.1:8000", "/" + uuid)

    parsed = urlparse(file_url)

    if parsed.scheme != "http":
        raise ValueError("Invalid sheme")
    if parsed.hostname != "127.0.0.1":
        raise ValueError("Invalid host")
    if parsed.port != 8000:
        raise ValueError("Invalid port")

    return file_url

if __name__ == '__main__':
    app.run()
