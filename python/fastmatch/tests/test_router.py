from __future__ import annotations

import fastmatch


def test_router_matches_path_parameters() -> None:
    router = fastmatch.Router([("GET", "/users/:id:int", 1)])
    route_id, params = router.lookup("GET", None, "/users/123")
    assert route_id == 1
    assert params == [("id", "123")]
