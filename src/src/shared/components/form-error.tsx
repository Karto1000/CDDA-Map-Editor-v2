import React from "react";
import "./form-error.scss"
import {DeepRequired, FieldErrorsImpl} from "react-hook-form";
import Icon, {IconName} from "./icon.js";

export type FormErrorProps<T> = {
    errors: Partial<FieldErrorsImpl<DeepRequired<T>>>
}

export function FormError<T>({errors}: FormErrorProps<T>) {
    const hasErrors = Object.keys(errors).length > 0;

    return hasErrors && (
        <div className={"form-error"}>
            <Icon name={IconName.ErrorMedium} width={24} height={24}/>
            <p>{errors[Object.keys(errors)[0]].message}</p>
        </div>
    )
}