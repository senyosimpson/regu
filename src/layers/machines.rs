use std::sync::Arc;
use std::task::{Context, Poll};

use tower::Service;

use crate::regu::{Origin, Store};
use crate::request::Request;

pub struct MachineHandler<S> {
    service: S,
    client: MachinesClient,
    store: Arc<Store>,
}

pub struct MachinesClient;

pub enum MachineState {
    Started,
    Stopped,
}

impl<S> Service<Request> for MachineHandler<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let origin = request.state.get::<Origin>().unwrap();
        // Check the state of the machine in the store
        if let Some(machine) = self.store.machines.get() {
            if matches!(machine.state, MachineState::Stopped) {
                self.client.start(machine.id).await; // what happens when this fails?
            }
        }

        self.service.call(request)
    }
}
