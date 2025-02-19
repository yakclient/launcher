import styles from "./authentication.module.sass"
import bg_png from "../../public/icons/login_bg.png"
import Image from "next/image";
import {Alert, Button} from "react-bootstrap";
import {invoke} from "@tauri-apps/api/core";
import {Alerts} from "@/pages/_app";
import {useRouter} from "next/router";
import {ReactNode, useContext, useEffect, useState} from "react";

const RefreshingCredentials: React.FC = () => {
    return <div className={styles.centered}>
        <h2>Refreshing credentials, please wait a moment...</h2>
    </div>
}

const AuthenticationProcessing: React.FC = () => {
    return <div className={styles.centered}>
        <h2>Processing authentication, one second please</h2>
    </div>
}

const SuccessfullyAuthenticated: React.FC = () => {
    return <div className={styles.centered}>
        <h2>Authentication successful</h2>
    </div>
}

enum AuthState {
    RefreshingCredentials,
    PromptLogin,
    // LoginAuthenticating,
    AuthenticationProcessing,
    SuccessfullyAuthenticated
}

const Authentication: React.FC = () => {
    const router = useRouter();
    const [authState, setAuthState] = useState(AuthState.RefreshingCredentials)
    const addAlert = useContext(Alerts)

    useEffect(() => {
        (window as any).noAuth = (): void => {
            invoke("use_no_auth").then(() => {
                router.push("/home")
            })
        };

        return () => {
            delete (window as any).myFunction;
        };
    })

    useEffect(() => {
        switch (authState) {
            case AuthState.RefreshingCredentials:
                invoke("do_ms_refresh").then(() => {
                    setAuthState(AuthState.SuccessfullyAuthenticated)
                }).catch((e) => {
                    setAuthState(AuthState.PromptLogin)
                })

                break
            case AuthState.SuccessfullyAuthenticated:
                addAlert(
                    "dark",
                    <>
                        <Alert.Heading>Success!</Alert.Heading>
                        <hr/>
                        You&apos;ve been authenticated.
                    </>
                )

                setTimeout(() => {
                    router.push("/home")
                        .then(() => {
                        })
                }, 500)
                break
            case AuthState.PromptLogin:
                break

        }
    }, [authState]);

    const showAuthState = (): ReactNode => {
        switch (authState) {

            case AuthState.PromptLogin:
                return <div id={styles.login} className={styles.centered}>
                    <h2>Login with Microsoft</h2>
                    <Button
                        as={"button"}
                        variant="success"
                        onClick={() => {
                            setAuthState(AuthState.AuthenticationProcessing)
                            invoke("microsoft_login")
                                .then(() => {
                                    setAuthState(AuthState.SuccessfullyAuthenticated)
                                })
                                .catch((reason) => {
                                    setAuthState(AuthState.PromptLogin)

                                    addAlert(
                                        "danger",
                                        <>
                                            <Alert.Heading>Failed to authenticate!</Alert.Heading>
                                            <hr/>
                                            {reason.toString()}
                                        </>
                                    )
                                })
                        }}
                    >Login</Button>
                </div>
            case AuthState.RefreshingCredentials:
                return <RefreshingCredentials></RefreshingCredentials>
            case AuthState.AuthenticationProcessing:
                return <AuthenticationProcessing/>
            case AuthState.SuccessfullyAuthenticated:
                return <SuccessfullyAuthenticated/>
        }
    }

    return (
        <div id={styles.container}>
            <div id={styles.bg}>
                <Image
                    src={bg_png}
                    alt={"Background"}
                    width={500}
                    height={800}
                    className={styles.title_image}
                />
            </div>
            <div id={styles.title} className={styles.centered}>
                <h1>YakClient</h1>
            </div>
            {
                showAuthState()
            }
        </div>
    )
}

export default Authentication;