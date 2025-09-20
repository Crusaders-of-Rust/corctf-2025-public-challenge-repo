package main

import (
	"embed"
	"encoding/base64"
	"fmt"
	"log"
	"net/http"
	"sync"

	"github.com/wailsapp/wails/v3/pkg/application"
	"github.com/wailsapp/wails/v3/pkg/events"
)

//go:embed assets
var assets embed.FS

var update = `<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>england</title>
</head>
<body>
    <form>
		<input name="video">
		<button>update video!</button>
	</form>
</body>
</html>`

var window application.Window
var updateMu sync.Mutex

func show(id string) {
	encoded := base64.StdEncoding.EncodeToString([]byte("https://youtube.com/embed/" + id))
	window.ExecJS(fmt.Sprintf("window.postMessage(atob('%s'))", encoded))
}

func server() {
	http.HandleFunc("/status", func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte("UP"))
	})
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		video := r.URL.Query().Get("video")
		if video == "" {
			_, _ = w.Write([]byte(update))
			return
		}

		updateMu.Lock()
		show(video)
		updateMu.Unlock()

		_, _ = w.Write([]byte("updated video being displayed!"))
	})
	_ = http.ListenAndServe(":8080", nil)
}

func main() {
	app := application.New(application.Options{
		Name:        "england",
		Description: "god save the king!",
		Services:    []application.Service{},
		ShouldQuit: func() bool {
			return true
		},
		Mac: application.MacOptions{
			ApplicationShouldTerminateAfterLastWindowClosed: true,
		},
		Assets: application.AssetOptions{
			Handler: application.AssetFileServerFS(assets),
		},
	})

	window = app.Window.New()
	serverStarted := false
	window.OnWindowEvent(events.Common.WindowRuntimeReady, func(event *application.WindowEvent) {
		if !serverStarted {
			log.Println("runtime ready")
			serverStarted = true
			go server()
		}
	})

	err := app.Run()
	if err != nil {
		log.Fatal(err)
	}
}
