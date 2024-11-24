module Main exposing (..)


import Browser
import Browser.Navigation as Nav
import Html exposing (Html, a, code, div, h1, h3, li, text, ul)
import Html.Attributes exposing (href)
import Url exposing (Url)
import Url.Parser as P exposing (Parser, (</>), (<?>), s, top)

-- MAIN
main : Program () Model Msg
main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        , onUrlRequest = UrlRequest
        , onUrlChange = UrlChange
        }

-- MODEL
type alias Model =
    { current: Maybe Route
    , key : Nav.Key
    }

init : () -> Url -> Nav.Key -> ( Model, Cmd Msg )
init _ url key =
    ( Model (P.parse routeParser url) key
    , Cmd.none
    )
    
-- URL PARSING
type Route
    = Home
    | Excercise String
    | Wod String


routeParser : Parser (Route -> a) a
routeParser =
    P.oneOf
        [ P.map Home top
        , P.map Excercise (s "excercise" </> P.string)
        , P.map Wod (s "wod" </> P.string)
        ]

-- UPDATE
type Msg
    = UrlChange Url
    | UrlRequest Browser.UrlRequest

update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        UrlChange url ->
            ( { model | current = P.parse routeParser url }
            , Cmd.none
            )

        UrlRequest request ->
            case request of
                Browser.Internal url ->
                    ( model
                    , Nav.pushUrl model.key (Url.toString url)
                    )

                Browser.External url ->
                    ( model
                    , Nav.load url
                    )

-- SUBSCRIPTIONS
subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none

-- VIEW
view : Model -> Browser.Document Msg
view model =
    Browser.Document "Gym Log"
        [ div []
            [ h1 [] [ text "Navigation" ]
            , ul [] (List.map viewLink [ "/", "/excercise/list", "/excercise/create", "/wod/create", "/wod/start"])
            , h1 [] [ text "Current content" ]
            , div [] [viewRoute model.current]
            ]
        ]

viewLink : String -> Html Msg
viewLink url =
    li [] [ a [ href url ] [ text url ] ]

viewRoute : Maybe Route -> Html msg
viewRoute maybeRoute =
    case maybeRoute of
        Nothing ->
            li [] [ code [] [ text "Well this is embarassing. This site is nowhere to be found :()" ] ]

        Just route ->
            case route of
                Home -> text "Ahaha"
                Excercise excs -> text ("Excercise verb: " ++ excs)
                Wod wod -> text ("Wod verb: " ++ wod)
