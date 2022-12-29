

function fetchOrders(params) {
    const myRequest = new Request("orders?" + new URLSearchParams(params));
    return fetch(myRequest)
        .then(response => response.json());
}