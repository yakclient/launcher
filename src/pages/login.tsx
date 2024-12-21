import styles from "./login.module.sass"
import bg_png from "../../public/icons/login_bg.png"
import Image from "next/image";
import {Alert, Button} from "react-bootstrap";
import {invoke} from "@tauri-apps/api/core";
import {Alerts} from "@/pages/_app";
import {useRouter} from "next/router";
import {useEffect} from "react";

const Login: React.FC = () => {
    const router = useRouter();

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

    return (
        <Alerts.Consumer>
            {addAlert =>
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
                    <div id={styles.login} className={styles.centered}>
                        <h2>Login with Microsoft</h2>
                        {/*<Button*/}
                        {/*    as={"button"}*/}
                        {/*    onClick={() => {*/}
                        {/*        invoke("use_no_auth")*/}
                        {/*            .then(() => {*/}
                        {/*                addAlert(*/}
                        {/*                    "success",*/}
                        {/*                    <>*/}
                        {/*                        <Alert.Heading>Success!</Alert.Heading>*/}
                        {/*                        <hr/>*/}
                        {/*                        You&apos;ve been authenticated.*/}
                        {/*                    </>*/}
                        {/*                )*/}
                        {/*                router.push("/home")*/}
                        {/*            })*/}
                        {/*    }}>*/}
                        {/*    Continue unauthenticated*/}
                        {/*</Button>*/}
                        <Button
                            as={"button"}
                            variant="success"
                            onClick={() => {
                                invoke("microsoft_login")
                                    .then(() => {
                                        addAlert(
                                            "dark",
                                            <>
                                                <Alert.Heading>Success!</Alert.Heading>
                                                <hr/>
                                                You&apos;ve been authenticated.
                                            </>
                                        )
                                        router.push("/home")
                                    })
                                    .catch((reason) => {
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
                </div>
            }
        </Alerts.Consumer>

    )
}

export default Login;