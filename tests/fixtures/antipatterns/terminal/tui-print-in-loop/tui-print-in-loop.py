from textual.app import App
from textual.worker import work

class Demo(App):
    def on_mount(self) -> None:
        print("hello")  # flag
        self.log("hello")  # pass

    @work(thread=True)
    def fetch(self) -> None:
        print("worker output")  # pass

def main() -> None:
    print("outside the app")  # pass
