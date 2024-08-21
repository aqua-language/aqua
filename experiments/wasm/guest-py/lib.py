import component


class Component(component.Component):
    def print(self) -> None:
        print("Hello, World!")

    def hello(self) -> str:
        return "Hello, World!"
