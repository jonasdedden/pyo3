from .pyclasses import EmptyClass
from datetime import date

def cross_module_imports(_a: EmptyClass) -> None: ...
def with_custom_type_annotations(
    a: list[int], *_args: str, _b: int | None = None, **_kwargs: bool
) -> int: ...
def with_imported_type_annotations(when: date) -> date | None:
    """
    Annotations naming something outside `builtins` have to be imported by the stub
    """
