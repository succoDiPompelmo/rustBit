use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

pub async fn call() {
    let mut client = GreeterClient::connect("http://0.0.0.0:50051")
        .await
        .unwrap();

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await.unwrap();

    println!("RESPONSE={:?}", response);
}
