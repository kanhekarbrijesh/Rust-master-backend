# setup rust axum backend boilerplate with crate 'axum' and 'tokio'

# setup first get 'hello world' endpoint 

# setup app centralized configurations which loads different configs for each enviroment ( local , dev , stage , prod) , use crate 'dotenvy' to load env's

# setup logging services with crate 'tracing-subscriber' and 'tracing'

# learn how routing works with "axum::{Router,route}"  : how to inititialize it , and how to use ( route , next , merge ) , and also how to use middleware

# setup appState to store data and infrastructure across whole app , and learn how to use it by passing it in router::new().withState(AppState), and use it anywhere inside route with (State(state):State(AppState))

# mongodb crud ( start commit id : 91a440e1234dcbd8ef21a8f19de5646e3c476902 | d38a6a7 , end commit id : 1f9838627377d36c0cdafb13a82c8245c2f44b80 | 1f98386  )

# postgres crud ( start commit id : , end commit id :  )



